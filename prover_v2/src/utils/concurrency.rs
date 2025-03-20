use anyhow::{Context, Result};
use redis::{AsyncCommands, Client, aio::ConnectionManager};
use serde_json;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, timeout};

use zkm2_core_executor::ExecutionRecord as Deferred;
use zkm2_stark::PublicValues;

type State = PublicValues<u32, u32>;

const MAX_WAIT: Duration = Duration::from_secs(600);

#[derive(Clone)]
pub struct DistributedLock {
    pub client: Client,
    pub turn_key: String,
    pub turn_channel: String,
    pub state_key: String,
    pub deferred_key: String,
    connection_pool: Arc<RwLock<ConnectionManager>>, // Efficient pooled connections
}

impl DistributedLock {
    /// Creates a new distributed lock with Redis connection pooling.
    pub async fn new(
        redis_url: &str,
        turn_key: &str,
        turn_channel: &str,
        state_key: &str,
        deferred_key: &str,
    ) -> Result<Self> {
        let client = Client::open(redis_url)?;
        let connection_manager = ConnectionManager::new(client.clone()).await?;

        let sync = DistributedLock {
            client,
            turn_key: turn_key.to_string(),
            turn_channel: turn_channel.to_string(),
            state_key: state_key.to_string(),
            deferred_key: deferred_key.to_string(),
            connection_pool: Arc::new(RwLock::new(connection_manager)),
        };

        sync.initialize_turn().await?;
        Ok(sync)
    }

    /// Initializes the turn value in Redis if not already set.
    async fn initialize_turn(&self) -> Result<()> {
        let mut conn = self.connection_pool.write().await;
        let current_turn: Option<usize> = conn.get(&self.turn_key).await?;
        if current_turn.is_none() {
            let _: () = conn.set(&self.turn_key, 0).await?;
        }
        Ok(())
    }

    /// Waits for a specific turn with a timeout to prevent deadlocks.
    pub async fn wait_for_turn(&self, my_turn: usize) -> Result<()> {
        let mut conn = self.connection_pool.write().await;
        let mut interval = tokio::time::interval(Duration::from_millis(100));

        loop {
            let current_turn: usize = conn.get(&self.turn_key).await.unwrap_or(0);
            if current_turn == my_turn {
                return Ok(());
            }

            if timeout(MAX_WAIT, interval.tick()).await.is_err() {
                return Err(anyhow::anyhow!("Timeout waiting for turn {}", my_turn));
            }
        }
    }

    /// Advances the turn and notifies subscribers.
    pub async fn advance_turn(self) -> Result<()> {
        let mut conn = self.connection_pool.write().await;
        let new_turn: usize = conn.incr(&self.turn_key, 1).await?;
        let _: () = conn.publish(&self.turn_channel, new_turn).await?;

        drop(conn);
        Ok(())
    }

    /// Gets the current state from Redis.
    pub async fn get_state(&self) -> Result<State> {
        let mut conn = self.connection_pool.write().await;
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

    /// Sets a new state in Redis.
    pub async fn set_state(&self, state: &State) -> Result<()> {
        let mut conn = self.connection_pool.write().await;
        let serialized_data = bincode::serialize(state)?;
        let _: () = conn.set(&self.state_key, serialized_data).await?;

        Ok(())
    }

    /// Gets the deferred value from Redis.
    pub async fn get_deferred(&self) -> Result<Deferred> {
        let mut conn = self.connection_pool.write().await;
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

    /// Sets a new deferred value in Redis.
    pub async fn set_deferred(&self, deferred: &Deferred) -> Result<()> {
        let mut conn = self.connection_pool.write().await;
        let serialized_data = bincode::serialize(deferred)?;
        let _: () = conn.set(&self.deferred_key, serialized_data).await?;
        Ok(())
    }
}
