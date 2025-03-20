use futures::StreamExt;
use redis::{AsyncCommands, Client};
use serde_json;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::time::Duration;
use anyhow::Result;

use zkm2_core_executor::ExecutionRecord;
use zkm2_stark::PublicValues;

#[derive(Clone)]
pub struct DistributedTurnBasedSync {
    pub client: Client,
    pub turn_key: String,
    pub turn_channel: String,
    pub state_key: String,
    pub deferred_key: String,

    /// Local cache, optional; if you want everything to go through Redis, you can remove this.
    shared_turn_cache: Arc<Mutex<usize>>,
    stop_subscribe_flag: Arc<RwLock<bool>>,
}

impl DistributedTurnBasedSync {
    /// Establish a connection to Redis and initialize `turn` to 0 if it does not exist.
    /// TODO: use a connection pool
    pub async fn new(
        redis_url: &str,
        turn_key: &str,
        turn_channel: &str,
        state_key: &str,
        deferred_key: &str,
    ) -> redis::RedisResult<Self> {
        let client = Client::open(redis_url)?;
        let mut conn = client.get_async_connection().await?;

        // If `turn_key` does not exist, initialize it to 0.
        let exists: bool = redis::cmd("EXISTS")
            .arg(turn_key)
            .query_async(&mut conn)
            .await?;
        if !exists {
            redis::cmd("SET")
                .arg(turn_key)
                .arg(0)
                .query_async::<()>(&mut conn)
                .await?;
        }

        Ok(DistributedTurnBasedSync {
            client,
            turn_key: turn_key.to_string(),
            turn_channel: turn_channel.to_string(),
            state_key: state_key.to_string(),
            deferred_key: deferred_key.to_string(),
            shared_turn_cache: Arc::new(Mutex::new(0)),
            stop_subscribe_flag: Arc::new(RwLock::new(false)),
        })
    }

    /// Subscribe to Redis `turn_channel` in the background to synchronize the latest turn.
    /// If you don't want to rely on Pub/Sub, you can use polling with GET instead.
    pub async fn start_subscribe_loop(&self) {
        let mut pubsub_conn = match self.client.get_async_connection().await {
            Ok(c) => c.into_pubsub(),
            Err(e) => {
                eprintln!("Failed to get pubsub connection: {:?}", e);
                return;
            }
        };

        if let Err(e) = pubsub_conn.subscribe(&self.turn_channel).await {
            eprintln!("Subscribe error: {:?}", e);
            return;
        }

        let shared_turn_cache = self.shared_turn_cache.clone();
        let stop_flag = self.stop_subscribe_flag.clone();

        // Spawn an async task to continuously receive messages.
        tokio::spawn(async move {
            while let Some(msg) = pubsub_conn.on_message().next().await {
                // Check if we need to stop.
                {
                    let read_stop = stop_flag.read().await;
                    if *read_stop {
                        println!("Stop subscribe loop");
                        break;
                    }
                }

                let payload: String = match msg.get_payload() {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("Failed to get payload: {:?}", e);
                        continue;
                    }
                };

                // Parse `turn=XXX`.
                if let Some(stripped) = payload.strip_prefix("turn=") {
                    if let Ok(new_turn) = stripped.parse::<usize>() {
                        let mut turn_cache = shared_turn_cache.lock().await;
                        *turn_cache = new_turn;
                    }
                }
            }
        });
    }

    /// Stop the subscription loop.
    pub async fn stop_subscribe(&self) {
        let mut flag = self.stop_subscribe_flag.write().await;
        *flag = true;
    }

    /// Block (asynchronously) until the current turn == `my_turn`.
    pub async fn wait_for_turn(&self, my_turn: usize) -> redis::RedisResult<()> {
        let mut conn = self.client.get_async_connection().await?;

        loop {
            // First check the local cache.
            {
                let local_turn = *self.shared_turn_cache.lock().await;
                if local_turn == my_turn {
                    return Ok(());
                }
            }

            // If `local_turn` is inconsistent, read from Redis again to prevent lost Pub/Sub messages.
            let actual_turn: usize = redis::cmd("GET")
                .arg(&self.turn_key)
                .query_async(&mut conn)
                .await
                .unwrap_or(0);

            {
                let mut local_turn_mut = self.shared_turn_cache.lock().await;
                *local_turn_mut = actual_turn;
            }

            if actual_turn == my_turn {
                return Ok(());
            }

            // Wait for a short time before retrying (simulating condvar wait).
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// Atomically increment `turn` and publish it.
    pub async fn advance_turn(&self) -> redis::RedisResult<usize> {
        let mut conn = self.client.get_async_connection().await?;
        // Atomic increment.
        let new_turn: usize = redis::cmd("INCR")
            .arg(&self.turn_key)
            .query_async(&mut conn)
            .await?;

        // Update local cache.
        {
            let mut local_turn = self.shared_turn_cache.lock().await;
            *local_turn = new_turn;
        }

        // Publish message "turn=xxx".
        let msg = format!("turn={}", new_turn);
        let _: () = redis::cmd("PUBLISH")
            .arg(&self.turn_channel)
            .arg(msg)
            .query_async(&mut conn)
            .await?;

        Ok(new_turn)
    }
}


type State = PublicValues<u32, u32>;
type Deferred = ExecutionRecord;
impl DistributedTurnBasedSync {
    /// 获取当前存储在 Redis 中的 state
    pub async fn get_state(&self) -> Result<State> {
        let mut conn = self.client.get_async_connection().await?;
        let raw_data: Option<Vec<u8>> = conn.get(&self.state_key).await?;

        if let Some(data) = raw_data {
            match bincode::deserialize::<State>(&data) {
                Ok(state) => Ok(state),
                Err(e) => {
                    tracing::error!("Failed to deserialize: {:?}", e);
                    Ok(Default::default())
                }
            }
        } else {
            Ok(Default::default())
        }
    }

    /// 设置 Redis 中的 state
    pub async fn set_state(&self, new_state: &State) -> Result<()> {
        let serialized_data = bincode::serialize(new_state)?;
        let mut conn = self.client.get_async_connection().await?;
        conn.set(&self.state_key, serialized_data).await?;

        Ok(())
    }

    /// 获取 deferred
    pub async fn get_deferred(&self) -> Result<Deferred> {
        let mut conn = self.client.get_async_connection().await?;
        let raw_data: Option<Vec<u8>> = conn.get(&self.deferred_key).await?;

        if let Some(data) = raw_data {
            match bincode::deserialize::<Deferred>(&data) {
                Ok(deferred) => Ok(deferred),
                Err(e) => {
                    tracing::error!("Failed to deserialize: {:?}", e);
                    Ok(Default::default())
                }
            }
        } else {
            Ok(Default::default())
        }
    }

    /// 设置 deferred
    pub async fn set_deferred(&self, new_deferred: &Deferred) -> Result<()> {
        let serialized_data = bincode::serialize(new_deferred)?;
        let mut conn = self.client.get_async_connection().await?;
        conn.set(&self.deferred_key, serialized_data).await?;

        Ok(())
    }
}
//
// // /// 演示如何在多节点里“按轮次”更新 `state` 和 `deferred`。
// // /// 假设这是节点 A 的逻辑。
// // pub async fn do_something_with_sync(sync: &DistributedTurnBasedSync, my_turn: usize) {
// //     // 等到我这个节点的轮次
// //     sync.wait_for_turn(my_turn).await.unwrap();
// //
// //     // 读 state
// //     let mut s = sync.get_state().await.unwrap();
// //     // 读 deferred
// //     let mut d = sync.get_deferred().await.unwrap();
// //
// //     // 假设做一些业务操作
// //     s.counter += 1;
// //     s.description = format!("updated by node A on turn {}", my_turn);
// //     d.pending_records.push(format!("append record by node A, turn={}", my_turn));
// //     d.last_update_ts = chrono::Utc::now().timestamp() as u64;
// //
// //     // 写回 Redis
// //     sync.set_state(&s).await.unwrap();
// //     sync.set_deferred(&d).await.unwrap();
// //
// //     // 完成后，通知进入下一轮
// //     sync.advance_turn().await.unwrap();
// // }
//
// #[test]
// fn test_distributed_turn_based_sync() {
//     println!("test_distributed_turn_based_sync");
// }

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use anyhow::{Context, Result};
use deadpool_redis::{Config, Connection, Pool, Runtime};
use redis::{AsyncCommands, RedisResult};
use serde::{de::DeserializeOwned, Serialize};
use tokio::sync::watch;
use tokio::time::sleep;
use tracing::{debug, error, info, instrument};
use zkm2_core_executor::ExecutionRecord;
use zkm2_stark::PublicValues;

/// 生产级分布式轮次同步器
#[derive(Clone)]
pub struct DistributedTurnBasedSync {
    // Redis 连接池
    redis_pool: Pool,

    // Redis 键配置
    turn_key: String,
    turn_channel: String,
    state_key: String,
    deferred_key: String,

    // 本地原子缓存
    local_turn: Arc<AtomicUsize>,

    // 订阅控制
    stop_tx: watch::Sender<bool>,
}

impl DistributedTurnBasedSync {
    /// 初始化同步器（带连接池和自动重试）
    pub async fn new(
        redis_url: &str,
        turn_key: &str,
        turn_channel: &str,
        state_key: &str,
        deferred_key: &str,
    ) -> Result<Self> {
        // 1. 初始化连接池
        let cfg = Config::from_url(redis_url);
        let pool = cfg.create_pool(Some(Runtime::Tokio))?;

        // 2. 初始化 Redis 状态
        let mut conn = Self::get_conn_with_retry(&pool).await?;
        let initial_turn: usize = conn.get(turn_key).await.unwrap_or(0);
        conn.set_nx::<_, _, ()>(turn_key, initial_turn).await?;

        // 3. 初始化本地状态
        let local_turn = Arc::new(AtomicUsize::new(initial_turn));

        // 4. 订阅控制通道
        let (stop_tx, _) = watch::channel(false);

        Ok(Self {
            redis_pool: pool,
            turn_key: turn_key.to_string(),
            turn_channel: turn_channel.to_string(),
            state_key: state_key.to_string(),
            deferred_key: deferred_key.to_string(),
            local_turn,
            stop_tx,
        })
    }

    /// 带指数退避的连接获取
    async fn get_conn_with_retry(pool: &Pool) -> Result<Connection> {
        let mut retries = 0;
        loop {
            match pool.get().await {
                Ok(conn) => return Ok(conn),
                Err(e) if retries < 5 => {
                    let delay = 2u64.pow(retries) * 100;
                    error!("Failed to get connection (retry {}): {:?}", retries, e);
                    sleep(Duration::from_millis(delay)).await;
                    retries += 1;
                }
                Err(e) => return Err(e).context("Failed to get Redis connection after retries"),
            }
        }
    }

    /// 启动订阅循环（自动重连）
    pub async fn start_subscription(&self) {
        let pool = self.redis_pool.clone();
        let turn_channel = self.turn_channel.clone();
        let local_turn = self.local_turn.clone();
        let mut stop_rx = self.stop_tx.subscribe();

        tokio::spawn(async move {
            loop {
                // 获取连接（带重试）
                let mut conn = match Self::get_conn_with_retry(&pool).await {
                    Ok(c) => c,
                    Err(e) => {
                        error!("Subscription connection failed: {:?}", e);
                        sleep(Duration::from_secs(5)).await;
                        continue;
                    }
                };

                // 创建 PubSub 连接
                let mut pubsub = match conn.into_pubsub() {
                    Ok(p) => p,
                    Err(e) => {
                        error!("Failed to create pubsub: {:?}", e);
                        continue;
                    }
                };

                // 订阅频道
                if let Err(e) = pubsub.subscribe(&turn_channel).await {
                    error!("Subscribe failed: {:?}", e);
                    continue;
                }

                // 消息处理循环
                let mut pubsub_stream = pubsub.on_message();
                loop {
                    tokio::select! {
                        // 处理消息
                        msg = pubsub_stream.next() => {
                            if let Some(msg) = msg {
                                if let Ok(payload) = msg.get_payload::<String>() {
                                    if let Some(turn_str) = payload.strip_prefix("turn=") {
                                        if let Ok(turn) = turn_str.parse::<usize>() {
                                            local_turn.store(turn, Ordering::SeqCst);
                                            debug!("Updated local turn: {}", turn);
                                        }
                                    }
                                }
                            }
                        }

                        // 检查停止信号
                        _ = stop_rx.changed() => {
                            if *stop_rx.borrow() {
                                info!("Stopping subscription loop");
                                return;
                            }
                        }
                    }
                }
            }
        });
    }

    /// 等待轮到指定轮次（带混合策略）
    #[instrument(skip(self))]
    pub async fn wait_for_turn(&self, expected_turn: usize) -> Result<()> {
        let mut conn = Self::get_conn_with_retry(&self.redis_pool).await?;
        let mut retries = 0;

        loop {
            // 优先检查本地缓存
            let current = self.local_turn.load(Ordering::SeqCst);
            if current == expected_turn {
                return Ok(());
            }

            // 混合策略：偶尔直接查询 Redis
            if retries % 5 == 0 {
                match conn.get::<_, usize>(&self.turn_key).await {
                    Ok(redis_turn) => {
                        self.local_turn.store(redis_turn, Ordering::SeqCst);
                        if redis_turn == expected_turn {
                            return Ok(());
                        }
                    }
                    Err(e) => error!("Redis query failed: {:?}", e),
                }
            }

            // 动态等待策略
            let delay = Duration::from_millis(50 * (retries + 1).min(10));
            sleep(delay).await;
            retries += 1;
        }
    }

    /// 原子推进轮次（使用 Lua 脚本保证原子性）
    #[instrument(skip(self))]
    pub async fn advance_turn(&self) -> Result<usize> {
        let lua_script = redis::Script::new(
            r#"
            local new_turn = redis.call('INCR', KEYS[1])
            redis.call('PUBLISH', KEYS[2], 'turn='..new_turn)
            return new_turn
            "#
        );

        let mut conn = Self::get_conn_with_retry(&self.redis_pool).await?;

        let new_turn: usize = lua_script
            .key(&self.turn_key)
            .key(&self.turn_channel)
            .invoke_async(&mut conn)
            .await
            .context("Failed to execute advance turn script")?;

        self.local_turn.store(new_turn, Ordering::SeqCst);
        Ok(new_turn)
    }

    // 泛型化的状态存储方法
    async fn get_serialized<T: DeserializeOwned + Default>(&self, key: &str) -> Result<T> {
        let mut conn = Self::get_conn_with_retry(&self.redis_pool).await?;
        let raw: Option<Vec<u8>> = conn.get(key).await?;

        match raw {
            Some(data) => bincode::deserialize(&data)
                .map_err(|e| anyhow::anyhow!("Deserialization failed: {:?}", e)),
            None => Ok(T::default()),
        }
    }

    async fn set_serialized<T: Serialize>(&self, key: &str, value: &T) -> Result<()> {
        let mut conn = Self::get_conn_with_retry(&self.redis_pool).await?;
        let data = bincode::serialize(value)?;
        conn.set(key, data).await?;
        Ok(())
    }

    /// 获取状态（严格错误处理）
    pub async fn get_state(&self) -> Result<PublicValues<u32, u32>> {
        self.get_serialized(&self.state_key).await
    }

    /// 设置状态（带版本控制）
    pub async fn set_state(&self, state: &PublicValues<u32, u32>) -> Result<()> {
        self.set_serialized(&self.state_key, state).await
    }

    /// 获取 Deferred（严格错误处理）
    pub async fn get_deferred(&self) -> Result<ExecutionRecord> {
        self.get_serialized(&self.deferred_key).await
    }

    /// 设置 Deferred（带版本控制）
    pub async fn set_deferred(&self, deferred: &ExecutionRecord) -> Result<()> {
        self.set_serialized(&self.deferred_key, deferred).await
    }

    /// 优雅关闭
    pub async fn shutdown(&self) {
        let _ = self.stop_tx.send(true);
    }
}