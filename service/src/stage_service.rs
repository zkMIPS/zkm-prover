use common::tls::Config as TlsConfig;
use stage_service::stage_service_server::StageService;
use stage_service::{GenerateProofRequest, GenerateProofResponse};
use stage_service::{GetStatusRequest, GetStatusResponse};
use tokio::sync::Mutex;

use stage::tasks::Task;
use tokio::sync::mpsc;
use tokio::time;
use tonic::{Request, Response, Status};

use crate::config;
use crate::prover_client;
use common::file::{create_dir_all, read, write_file};
use prover::provers::{self};

#[allow(clippy::module_inception)]
pub mod stage_service {
    tonic::include_proto!("stage.v1");
}

use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    static ref GLOBAL_TASKMAP: Mutex<HashMap<String, i32>> = Mutex::new(HashMap::new());
}

pub struct StageServiceSVC {
    tls_config: Option<TlsConfig>,
}

impl StageServiceSVC {
    pub async fn new(config: config::RuntimeConfig) -> anyhow::Result<Self> {
        let tls_config = if config.ca_cert_path.is_some() {
            Some(
                TlsConfig::new(
                    config.ca_cert_path.unwrap(),
                    config.cert_path.unwrap(),
                    config.key_path.unwrap(),
                )
                .await?,
            )
        } else {
            None
        };
        Ok(StageServiceSVC { tls_config })
    }
}

#[tonic::async_trait]
impl StageService for StageServiceSVC {
    async fn get_status(
        &self,
        request: Request<GetStatusRequest>,
    ) -> tonic::Result<Response<GetStatusResponse>, Status> {
        // log::info!("{:?}", request);
        let taskmap = GLOBAL_TASKMAP.lock().await;
        let status = taskmap.get(&request.get_ref().proof_id);
        let mut response = stage_service::GetStatusResponse {
            proof_id: request.get_ref().proof_id.clone(),
            ..Default::default()
        };
        if let Some(status) = status {
            response.status = *status as u64;
        }
        Ok(Response::new(response))
    }

    async fn generate_proof(
        &self,
        request: Request<GenerateProofRequest>,
    ) -> tonic::Result<Response<GenerateProofResponse>, Status> {
        log::info!("{:?}", request.get_ref().proof_id);

        // check seg_size
        if !provers::valid_seg_size(request.get_ref().seg_size as usize) {
            let response = stage_service::GenerateProofResponse {
                proof_id: request.get_ref().proof_id.clone(),
                executor_error: stage_service::ExecutorError::Unspecified as u32,
                error_message: "invalid seg_size".to_string(),
                ..Default::default()
            };
            return Ok(Response::new(response));
        }

        let base_dir = config::instance().lock().unwrap().base_dir.clone();
        let dir_path = format!("{}/proof/{}", base_dir, request.get_ref().proof_id);
        create_dir_all(&dir_path)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let elf_path = format!("{}/elf", dir_path);
        write_file(&elf_path, &request.get_ref().elf_data)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let bolck_dir = format!("{}/0_{}", dir_path, request.get_ref().block_no);
        create_dir_all(&bolck_dir)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        for file_block_item in &request.get_ref().block_data {
            let bolck_path = format!("{}/{}", bolck_dir, file_block_item.file_name);
            write_file(&bolck_path, &file_block_item.file_content)
                .await
                .map_err(|e| Status::internal(e.to_string()))?;
        }

        let seg_path = format!("{}/segment", dir_path);
        create_dir_all(&seg_path)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let prove_path = format!("{}/prove", dir_path);
        create_dir_all(&prove_path)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let prove_proof_path = format!("{}/proof", prove_path);
        create_dir_all(&prove_proof_path)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let prove_pub_value_path = format!("{}/pub_value", prove_path);
        create_dir_all(&prove_pub_value_path)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let agg_path = format!("{}/aggregate", dir_path);
        create_dir_all(&agg_path)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let final_dir = format!("{}/final", dir_path);
        create_dir_all(&final_dir)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        let final_path = format!("{}/output", final_dir);

        {
            let mut taskmap = GLOBAL_TASKMAP.lock().await;
            taskmap.insert(
                request.get_ref().proof_id.clone(),
                stage_service::ExecutorError::Unspecified.into(),
            );
        }

        let generate_context = stage::contexts::GenerateContext::new(
            &request.get_ref().proof_id,
            &dir_path,
            &elf_path,
            &seg_path,
            &prove_path,
            &agg_path,
            &final_path,
            request.get_ref().block_no,
            request.get_ref().seg_size,
        );

        let mut stage = stage::stage::Stage::new(generate_context);
        let (tx, mut rx) = mpsc::channel(128);
        stage.dispatch().await;
        loop {
            println!("loop...");
            let split_task = stage.get_split_task();
            if let Some(split_task) = split_task {
                let tx = tx.clone();
                let tls_config = self.tls_config.clone();
                tokio::spawn(async move {
                    println!("send split task");
                    let response = prover_client::split(split_task, tls_config).await;
                    if let Some(split_task) = response {
                        tx.send(Task::Split(split_task)).await.unwrap();
                    }
                });
            }
            let prove_task = stage.get_prove_task();
            if let Some(prove_task) = prove_task {
                let tx = tx.clone();
                let tls_config = self.tls_config.clone();
                tokio::spawn(async move {
                    println!("send prove task");
                    let response = prover_client::prove(prove_task, tls_config).await;
                    if let Some(prove_task) = response {
                        tx.send(Task::Prove(prove_task)).await.unwrap();
                    }
                });
            }
            let agg_task = stage.get_agg_all_task();
            if let Some(agg_task) = agg_task {
                let tx = tx.clone();
                let tls_config = self.tls_config.clone();
                tokio::spawn(async move {
                    println!("send aggregate task");
                    let response = prover_client::aggregate_all(agg_task, tls_config).await;
                    if let Some(agg_task) = response {
                        tx.send(Task::Agg(agg_task)).await.unwrap();
                    }
                });
            }
            let final_task = stage.get_final_task();
            if let Some(final_task) = final_task {
                let tx = tx.clone();
                let tls_config = self.tls_config.clone();
                tokio::spawn(async move {
                    println!("send final task");
                    let response = prover_client::final_proof(final_task, tls_config).await;
                    if let Some(final_task) = response {
                        tx.send(Task::Final(final_task)).await.unwrap();
                    }
                });
            }

            tokio::select! {
                task = rx.recv() => {
                    if let Some(task) = task {
                        match task {
                            Task::Split(data) => {
                                stage.on_split_task(data);
                            },
                            Task::Prove(data) => {
                                stage.on_prove_task(data);
                            },
                            Task::Agg(data) => {
                                stage.on_agg_all_task(data);
                            },
                            Task::Final(data) => {
                                stage.on_final_task(data);
                            },
                        };
                    }
                },
                () = time::sleep(time::Duration::from_secs(1)) => {
                }
            };
            if stage.is_success() || stage.is_error() {
                break;
            }
            stage.dispatch().await;
        }

        let mut response = stage_service::GenerateProofResponse {
            proof_id: request.get_ref().proof_id.clone(),
            ..Default::default()
        };
        {
            let mut taskmap = GLOBAL_TASKMAP.lock().await;
            if stage.is_error() {
                response.executor_error = stage_service::ExecutorError::Error as u32;
                taskmap.insert(
                    request.get_ref().proof_id.clone(),
                    stage_service::ExecutorError::Error.into(),
                );
            } else {
                let result = read(&final_path).await.unwrap();
                response.result.clone_from(&result);
                response.executor_error = stage_service::ExecutorError::NoError as u32;
                taskmap.insert(
                    request.get_ref().proof_id.clone(),
                    stage_service::ExecutorError::NoError.into(),
                );
            }
        }

        log::info!("{}", stage.timecost_string());
        Ok(Response::new(response))
    }
}
