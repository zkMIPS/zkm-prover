use common::tls::Config as TlsConfig;
use stage_service::stage_service_server::StageService;
use stage_service::{GenerateProofRequest, GenerateProofResponse};
use stage_service::{GetStatusRequest, GetStatusResponse};
use std::sync::Mutex;

use prover::provers;
use std::fs;
use std::fs::File;
use std::io::Write;
use tonic::{Request, Response, Status};

use crate::stage_worker;
use crate::{config, database};

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
            response.status = task.status as u64;
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
        fs::create_dir_all(dir_path.clone())?;

        let elf_path = format!("{}/elf", dir_path);
        let mut file = File::create(elf_path.clone())?;
        file.write_all(&request.get_ref().elf_data)?;

        let bolck_dir = format!("{}/0_{}", dir_path, request.get_ref().block_no);
        fs::create_dir_all(bolck_dir.clone())?;

        for file_block_item in &request.get_ref().block_data {
            let bolck_path = format!("{}/{}", bolck_dir, file_block_item.file_name);
            let mut file = File::create(bolck_path)?;
            file.write_all(&file_block_item.file_content)?;
        }

        let seg_path = format!("{}/segment", dir_path);
        fs::create_dir_all(seg_path.clone())?;

        let prove_path = format!("{}/prove", dir_path);
        fs::create_dir_all(prove_path.clone())?;

        let prove_proof_path = format!("{}/proof", prove_path);
        fs::create_dir_all(prove_proof_path.clone())?;

        let prove_pub_value_path = format!("{}/pub_value", prove_path);
        fs::create_dir_all(prove_pub_value_path.clone())?;

        let agg_path = format!("{}/aggregate", dir_path);
        fs::create_dir_all(agg_path.clone())?;

        let final_dir = format!("{}/final", dir_path);
        fs::create_dir_all(final_dir.clone())?;
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
                stage_service::ExecutorError::Unspecified as i32,
                &serde_json::to_string(&generate_context).unwrap(),
            )
            .await;
        let response = stage_service::GenerateProofResponse {
            proof_id: request.get_ref().proof_id.clone(),
            executor_error: stage_service::ExecutorError::NoError as u32,
            ..Default::default()
        };
        Ok(Response::new(response))
    }
}
