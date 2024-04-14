use common::tls::Config as TlsConfig;
use stage_service::stage_service_server::StageService;
use stage_service::{GenerateProofRequest, GenerateProofResponse};
use stage_service::{GetStatusRequest, GetStatusResponse};
use std::sync::Mutex;

use tonic::{Request, Response, Status};

use crate::config;
use common::file;
use prover::provers;
use std::io::Write;

use crate::database;
use crate::stage_worker;

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
    db: database::Database,
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
        let database_url = config.database_url.as_str();
        let db = database::Database::new(database_url);
        sqlx::migrate!("./migrations").run(&db.db_pool).await?;
        let _ = stage_worker::start(tls_config.clone(), db.clone()).await;
        Ok(StageServiceSVC { db })
    }
}

#[tonic::async_trait]
impl StageService for StageServiceSVC {
    async fn get_status(
        &self,
        request: Request<GetStatusRequest>,
    ) -> tonic::Result<Response<GetStatusResponse>, Status> {
        let task = self.db.get_stage_task(&request.get_ref().proof_id).await;
        let mut response = stage_service::GetStatusResponse {
            proof_id: request.get_ref().proof_id.clone(),
            ..Default::default()
        };
        if let Ok(task) = task {
            response.status = task.status as u32;
            if let Some(result) = task.result {
                response.result = result.into_bytes();
            }
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
                status: stage_service::Status::Unspecified as u32,
                error_message: "invalid seg_size".to_string(),
                ..Default::default()
            };
            return Ok(Response::new(response));
        }

        let base_dir = config::instance().lock().unwrap().base_dir.clone();
        let dir_path = format!("{}/proof/{}", base_dir, request.get_ref().proof_id);
        file::new(&dir_path)
            .create_dir_all()
            .map_err(|e| Status::internal(e.to_string()))?;

        let elf_path = format!("{}/elf", dir_path);
        file::new(&elf_path)
            .write(&request.get_ref().elf_data)
            .map_err(|e| Status::internal(e.to_string()))?;

        let block_dir = format!("{}/0_{}", dir_path, request.get_ref().block_no);
        file::new(&block_dir)
            .create_dir_all()
            .map_err(|e| Status::internal(e.to_string()))?;

        for file_block_item in &request.get_ref().block_data {
            let block_path = format!("{}/{}", block_dir, file_block_item.file_name);
            file::new(&block_path)
                .write(&file_block_item.file_content)
                .map_err(|e| Status::internal(e.to_string()))?;
        }

        let seg_path = format!("{}/segment", dir_path);
        file::new(&seg_path)
            .create_dir_all()
            .map_err(|e| Status::internal(e.to_string()))?;

        let prove_path = format!("{}/prove", dir_path);
        file::new(&prove_path)
            .create_dir_all()
            .map_err(|e| Status::internal(e.to_string()))?;

        let prove_proof_path = format!("{}/proof", prove_path);
        file::new(&prove_proof_path)
            .create_dir_all()
            .map_err(|e| Status::internal(e.to_string()))?;

        let prove_pub_value_path = format!("{}/pub_value", prove_path);
        file::new(&prove_pub_value_path)
            .create_dir_all()
            .map_err(|e| Status::internal(e.to_string()))?;

        let agg_path = format!("{}/aggregate", dir_path);
        file::new(&agg_path)
            .create_dir_all()
            .map_err(|e| Status::internal(e.to_string()))?;

        let final_dir = format!("{}/final", dir_path);
        file::new(&final_dir)
            .create_dir_all()
            .map_err(|e| Status::internal(e.to_string()))?;
        let final_path = format!("{}/output", final_dir);

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

        let _ = self
            .db
            .insert_stage_task(
                &request.get_ref().proof_id,
                stage_service::Status::Computing as i32,
                &serde_json::to_string(&generate_context).unwrap(),
            )
            .await;
        let response = stage_service::GenerateProofResponse {
            proof_id: request.get_ref().proof_id.clone(),
            status: stage_service::Status::Computing as u32,
            ..Default::default()
        };
        Ok(Response::new(response))
    }
}
