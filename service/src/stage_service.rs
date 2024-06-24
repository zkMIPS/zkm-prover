use anyhow::Error;
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

use ethers::types::Signature;
use std::str::FromStr;

use crate::database;
use crate::metrics;
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
    fileserver_url: Option<String>,
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
        Ok(StageServiceSVC {
            db,
            fileserver_url: config.fileserver_url.clone(),
        })
    }

    pub fn valid_signature(&self, request: &GenerateProofRequest) -> Result<String, Error> {
        let sign_data = format!(
            "{}&{}&{}&{}",
            request.proof_id, request.block_no, request.seg_size, request.args
        );
        let signature = Signature::from_str(&request.signature)?;
        let recovered = signature.recover(sign_data)?;
        Ok(hex::encode(recovered))
    }
}

#[tonic::async_trait]
impl StageService for StageServiceSVC {
    async fn get_status(
        &self,
        request: Request<GetStatusRequest>,
    ) -> tonic::Result<Response<GetStatusResponse>, Status> {
        metrics::record_metrics("stage::get_status", || async {
            let task = self.db.get_stage_task(&request.get_ref().proof_id).await;
            let mut response = stage_service::GetStatusResponse {
                proof_id: request.get_ref().proof_id.clone(),
                ..Default::default()
            };
            if let Ok(task) = task {
                response.status = task.status as u32;
                if let Some(result) = task.result {
                    response.proof_with_public_inputs = result.into_bytes();
                }
                if let Some(fileserver_url) = &self.fileserver_url {
                    response.download_url = format!(
                        "{}/{}/final/proof_with_public_inputs.json",
                        fileserver_url,
                        request.get_ref().proof_id
                    );
                }
            }
            Ok(Response::new(response))
        })
        .await
    }

    async fn generate_proof(
        &self,
        request: Request<GenerateProofRequest>,
    ) -> tonic::Result<Response<GenerateProofResponse>, Status> {
        metrics::record_metrics("stage::generate_proof", || async {
            log::info!("{:?}", request.get_ref().proof_id);

            // check seg_size
            if !provers::valid_seg_size(request.get_ref().seg_size as usize) {
                let response = stage_service::GenerateProofResponse {
                    proof_id: request.get_ref().proof_id.clone(),
                    status: stage_service::Status::InvalidParameter as u32,
                    error_message: "invalid seg_size support [65536-262144]".to_string(),
                    ..Default::default()
                };
                return Ok(Response::new(response));
            }
            // check signature
            let user_address: String;
            match self.valid_signature(request.get_ref()) {
                Ok(address) => {
                    // check white list
                    let users = self.db.get_user(&address).await.unwrap();
                    log::info!(
                        "proof_id:{:?} address:{:?} exists:{:?}",
                        request.get_ref().proof_id,
                        address,
                        users.is_empty(),
                    );
                    if users.is_empty() {
                        let response = stage_service::GenerateProofResponse {
                            proof_id: request.get_ref().proof_id.clone(),
                            status: stage_service::Status::InvalidParameter as u32,
                            error_message: "permission denied".to_string(),
                            ..Default::default()
                        };
                        return Ok(Response::new(response));
                    }
                    user_address = users[0].address.clone();
                }
                Err(e) => {
                    let response = stage_service::GenerateProofResponse {
                        proof_id: request.get_ref().proof_id.clone(),
                        status: stage_service::Status::InvalidParameter as u32,
                        error_message: "invalid signature".to_string(),
                        ..Default::default()
                    };
                    log::warn!("{:?} invalid signature {:?}", request.get_ref().proof_id, e);
                    return Ok(Response::new(response));
                }
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
            let final_path = format!("{}/proof_with_public_inputs.json", final_dir);

            let generate_context = stage::contexts::GenerateContext::new(
                &request.get_ref().proof_id,
                &dir_path,
                &elf_path,
                &seg_path,
                &prove_path,
                &agg_path,
                &final_path,
                &request.get_ref().args,
                request.get_ref().block_no,
                request.get_ref().seg_size,
            );

            let _ = self
                .db
                .insert_stage_task(
                    &request.get_ref().proof_id,
                    &user_address,
                    stage_service::Status::Computing as i32,
                    &serde_json::to_string(&generate_context).unwrap(),
                )
                .await;
            let download_url = match &self.fileserver_url {
                Some(fileserver_url) => format!(
                    "{}/{}/final/proof_with_public_inputs.json",
                    fileserver_url,
                    request.get_ref().proof_id
                ),
                None => "".to_string(),
            };
            let response = stage_service::GenerateProofResponse {
                proof_id: request.get_ref().proof_id.clone(),
                status: stage_service::Status::Computing as u32,
                download_url,
                ..Default::default()
            };
            Ok(Response::new(response))
        })
        .await
    }
}
