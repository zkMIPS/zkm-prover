use crate::database;
use crate::database::StageTask;
use crate::prover_client;
use crate::stage::{
    stage::get_timestamp,
    stage::Stage,
    tasks::{
        Task, TASK_ITYPE_AGG, TASK_ITYPE_FINAL, TASK_ITYPE_PROVE, TASK_ITYPE_SPLIT,
        TASK_STATE_FAILED, TASK_STATE_SUCCESS,
    },
    GenerateTask,
};
use crate::TlsConfig;
use common::file;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::time;

use crate::proto::stage_service::{self, v1::Step};

macro_rules! save_task {
    ($task:ident, $db_pool:ident, $type:expr) => {
        if $task.state == TASK_STATE_FAILED || $task.state == TASK_STATE_SUCCESS {
            log::info!(
                "begin to save task: {:?}:{:?} type {:?} status {}",
                $task.proof_id,
                $task.task_id,
                $type,
                $task.state
            );
            // TODO: should remove the content from database, store it by FS.
            let content = serde_json::to_string(&$task).unwrap();
            let prove_task = database::ProveTask {
                id: $task.task_id,
                itype: $type,
                proof_id: $task.proof_id,
                status: $task.state as i32,
                node_info: $task.trace.node_info.clone(),
                content: Some(content),
                time_cost: ($task.trace.duration()) as i64,
                ..Default::default()
            };
            if let Err(e) = $db_pool.insert_prove_task(&prove_task).await {
                log::error!("save task error: {:?}", e)
            }
        }
    };
}

async fn run_stage_task(
    node_num: usize,
    mut task: StageTask,
    tls_config: Option<TlsConfig>,
    db: database::Database,
) {
    if let Some(context) = task.context {
        let task_decoded = serde_json::from_str::<GenerateTask>(&context);
        match task_decoded {
            Ok(generte_context) => {
                let mut check_at = get_timestamp();
                let mut stage = Stage::new(generte_context.clone());
                let (tx, mut rx) = tokio::sync::mpsc::channel(128);
                stage.dispatch();
                let mut interval = time::interval(time::Duration::from_millis(200));
                loop {
                    let current_step = stage.step;
                    match stage.step {
                        Step::InSplit => {
                            let split_task = stage.get_split_task();
                            if let Some(split_task) = split_task {
                                let tx = tx.clone();
                                let tls_config = tls_config.clone();
                                tokio::spawn(async move {
                                    let response =
                                        prover_client::split(split_task, tls_config).await;
                                    if let Some(split_task) = response {
                                        let _ = tx.send(Task::Split(split_task)).await;
                                    }
                                });
                            }
                        }
                        Step::InProve => {
                            let prove_task = stage.get_prove_task();
                            log::debug!("Step::InProve get_prove_task {:?}", prove_task.is_some());
                            if let Some(prove_task) = prove_task {
                                let tx = tx.clone();
                                let tls_config = tls_config.clone();
                                tokio::spawn(async move {
                                    let response =
                                        prover_client::prove(prove_task, tls_config).await;
                                    if let Some(prove_task) = response {
                                        let _ = tx.send(Task::Prove(prove_task)).await;
                                    }
                                });
                            }
                            // }
                            // Step::InAgg => {
                            //
                            if stage.count_unfinished_prove_tasks() < node_num {
                                let agg_task = stage.get_agg_task();
                                log::debug!("get_agg_task: {:?}", agg_task.is_some());
                                if let Some(agg_task) = agg_task {
                                    let tx = tx.clone();
                                    let tls_config = tls_config.clone();
                                    tokio::spawn(async move {
                                        let response =
                                            prover_client::aggregate(agg_task, tls_config).await;
                                        if let Some(agg_task) = response {
                                            let _ = tx.send(Task::Agg(agg_task)).await;
                                        }
                                    });
                                }
                            }
                        }
                        Step::InSnark => {
                            let snark_task = stage.get_snark_task();
                            if let Some(snark_task) = snark_task {
                                let tx = tx.clone();
                                let tls_config = tls_config.clone();
                                tokio::spawn(async move {
                                    let response =
                                        prover_client::snark_proof(snark_task, tls_config).await;
                                    if let Some(snark_task) = response {
                                        let _ = tx.send(Task::Snark(snark_task)).await;
                                    }
                                });
                            }
                        }
                        _ => {}
                    }
                    tokio::select! {
                        task = rx.recv() => {
                            if let Some(task) = task {
                                match task {
                                    Task::Split(mut data) => {
                                        stage.on_split_task(&mut data);
                                        save_task!(data, db, TASK_ITYPE_SPLIT);
                                    },
                                    Task::Prove(mut data) => {
                                        stage.on_prove_task(&mut data);
                                        save_task!(data, db, TASK_ITYPE_PROVE);
                                    },
                                    Task::Agg(mut data) => {
                                        stage.on_agg_task(&mut data);
                                        save_task!(data, db, TASK_ITYPE_AGG);
                                    },
                                    Task::Snark(mut data) => {
                                        stage.on_snark_task(&mut data);
                                        save_task!(data, db, TASK_ITYPE_FINAL);
                                    },
                                };
                            }
                        },
                        _ = interval.tick() => {
                        }
                    }
                    if stage.is_success() || stage.is_error() {
                        break;
                    }
                    stage.dispatch();
                    let ts_now = get_timestamp();
                    if check_at + 10 < ts_now || current_step != stage.step {
                        check_at = ts_now;
                        let rows_affected = db
                            .update_stage_task_check_at(
                                &task.id,
                                task.check_at as u64,
                                check_at,
                                stage.step.into(),
                            )
                            .await;
                        if let Ok(rows_affected) = rows_affected {
                            if rows_affected == 1 {
                                task.check_at = check_at as i64;
                            }
                        }
                    }
                }
                if stage.is_error() {
                    let get_status = || match stage.step {
                        Step::InSplit => stage_service::v1::Status::SplitError,
                        Step::InProve => stage_service::v1::Status::ProveError,
                        Step::InAgg => stage_service::v1::Status::AggError,
                        Step::InAggAll => stage_service::v1::Status::AggError,
                        Step::InSnark => stage_service::v1::Status::SnarkError,
                        _ => stage_service::v1::Status::InternalError,
                    };
                    let status = get_status();
                    db.update_stage_task(&task.id, status.into(), "")
                        .await
                        .unwrap();
                } else {
                    let result = if generte_context.execute_only || generte_context.composite_proof
                    {
                        vec![]
                    } else {
                        file::new(&generte_context.snark_path).read().unwrap()
                    };
                    db.update_stage_task(
                        &task.id,
                        stage_service::v1::Status::Success.into(),
                        &String::from_utf8(result).expect("Invalid UTF-8 bytes"),
                    )
                    .await
                    .unwrap();
                    log::info!("[stage] finished {:?} ", stage);
                }
            }
            Err(_) => {
                let _ = db
                    .update_stage_task(
                        &task.id,
                        stage_service::v1::Status::InternalError.into(),
                        "",
                    )
                    .await;
            }
        }
    }
}

async fn load_stage_task(node_num: usize, tls_config: Option<TlsConfig>, db: database::Database) {
    let store = Arc::new(Mutex::new(HashMap::new()));
    loop {
        let limit = 5;
        let status = stage_service::v1::Status::Computing.into();
        let check_at = get_timestamp();
        // FIXME: why do we just fetch the task in last 1 min?
        let result = db
            .get_incomplete_stage_tasks(status, (check_at - 60) as i64, limit)
            .await;
        match result {
            Ok(tasks) => {
                if tasks.is_empty() {
                    time::sleep(time::Duration::from_secs(1)).await;
                } else {
                    for mut task in tasks {
                        {
                            if store.lock().unwrap().contains_key(&task.id) {
                                continue;
                            }
                            let rows_affected = db
                                .update_stage_task_check_at(
                                    &task.id,
                                    task.check_at as u64,
                                    check_at,
                                    task.step,
                                )
                                .await;
                            if let Ok(rows_affected) = rows_affected {
                                if rows_affected == 1 {
                                    task.check_at = check_at as i64;
                                    store.lock().unwrap().insert(task.id.clone(), check_at);
                                    let store_arc = store.clone();
                                    let tls_config_copy = tls_config.clone();
                                    let db_copy = db.clone();
                                    tokio::spawn(async move {
                                        let id = task.id.clone();
                                        run_stage_task(node_num, task, tls_config_copy, db_copy)
                                            .await;
                                        store_arc.lock().unwrap().remove(&id);
                                    });
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                log::error!("{:?}", e);
                time::sleep(time::Duration::from_secs(10)).await;
            }
        }
    }
}

pub async fn start(
    node_num: usize,
    tls_config: Option<TlsConfig>,
    db: database::Database,
) -> anyhow::Result<bool> {
    tokio::spawn(async move {
        load_stage_task(node_num, tls_config, db).await;
    });
    Ok(true)
}
