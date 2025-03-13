use crate::proto::prover_service::v1::{
    get_status_response, prover_service_server::ProverService, AggregateRequest, AggregateResponse,
    GetStatusRequest, GetStatusResponse, GetTaskResultRequest, GetTaskResultResponse, ProveRequest,
    ProveResponse, Result, ResultCode, SnarkProofRequest, SnarkProofResponse, SplitElfRequest,
    SplitElfResponse,
};
use prover::contexts::{AggContext, ProveContext, SnarkContext};
use prover::executor::SplitContext;
use prover::pipeline::Pipeline;

use std::time::Instant;
use tonic::{Request, Response, Status};

use crate::proto::includes::v1::ProverVersion;
use crate::{config, metrics};

async fn run_back_task<
    T: Send + 'static,
    F: FnOnce() -> std::result::Result<T, String> + Send + 'static,
>(
    callable: F,
) -> std::result::Result<T, String> {
    let rt = tokio::runtime::Handle::current();
    let (tx, rx) = tokio::sync::oneshot::channel();
    let _ = rt
        .spawn_blocking(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(callable));
            let _ = tx.send(result);
        })
        .await;

    rx.await.unwrap().unwrap_or_else(|e| {
        let panic_message = if let Some(msg) = e.downcast_ref::<&str>() {
            msg.to_string()
        } else if let Some(msg) = e.downcast_ref::<String>() {
            msg.clone()
        } else {
            "Unknown panic".to_string()
        };

        log::error!("Task panicked: {}", panic_message);
        Err(panic_message) // Convert into a boxed error
    })
}

use std::sync::{Arc, Mutex};
#[derive(Default)]
pub struct ProverServiceSVC {
    pub config: config::RuntimeConfig,
    pipeline: Arc<Mutex<Pipeline>>,
}
impl ProverServiceSVC {
    pub fn new(config: config::RuntimeConfig) -> Self {
        let pipeline = Arc::new(Mutex::new(Pipeline::new(
            &config.base_dir,
            &config.get_proving_key_path(ProverVersion::Zkm.into()),
        )));
        Self { config, pipeline }
    }
}

macro_rules! on_done {
    ($result:ident, $resp:ident) => {
        match $result {
            Ok((done, _data)) => {
                if done {
                    $resp.result = Some(Result {
                        code: (ResultCode::Ok.into()),
                        message: "SUCCESS".to_string(),
                    });
                } else {
                    $resp.result = Some(Result {
                        code: (ResultCode::Busy.into()),
                        message: ("BUSY".to_string()),
                    });
                }
            }
            Err(e) => {
                $resp.result = Some(Result {
                    code: (ResultCode::InternalError.into()),
                    message: (e.to_string()),
                });
            }
        }
    };
}

#[tonic::async_trait]
impl ProverService for ProverServiceSVC {
    async fn get_status(
        &self,
        _request: Request<GetStatusRequest>,
    ) -> tonic::Result<Response<GetStatusResponse>, Status> {
        metrics::record_metrics("prover::get_status", || async {
            // log::info!("{:#?}", request);
            let mut response = GetStatusResponse::default();
            if self.pipeline.lock().is_err() {
                response.status = get_status_response::Status::Computing.into();
                return Ok(Response::new(response));
            }
            let success = self.pipeline.lock().unwrap().get_status();
            if success {
                response.status = get_status_response::Status::Idle.into();
            } else {
                response.status = get_status_response::Status::Computing.into();
            }
            Ok(Response::new(response))
        })
        .await
    }

    async fn get_task_result(
        &self,
        _request: Request<GetTaskResultRequest>,
    ) -> tonic::Result<Response<GetTaskResultResponse>, Status> {
        metrics::record_metrics("prover::get_task_result", || async {
            // log::info!("{:#?}", request);
            let response = GetTaskResultResponse::default();
            Ok(Response::new(response))
        })
        .await
    }

    async fn split_elf(
        &self,
        request: Request<SplitElfRequest>,
    ) -> tonic::Result<Response<SplitElfResponse>, Status> {
        metrics::record_metrics("prover::split_elf", || async {
            log::info!(
                "[split_elf] {}:{} start",
                request.get_ref().proof_id,
                request.get_ref().computed_request_id,
            );
            let start = Instant::now();
            let split_context = SplitContext::new(
                &request.get_ref().base_dir,
                &request.get_ref().elf_path,
                request.get_ref().block_no,
                request.get_ref().seg_size,
                &request.get_ref().seg_path,
                &request.get_ref().public_input_path,
                &request.get_ref().private_input_path,
                &request.get_ref().output_path,
                &request.get_ref().args,
                &request.get_ref().receipt_inputs_path,
            );

            let pipeline = self.pipeline.clone();
            let split_func = move || pipeline.lock().unwrap().split(&split_context);
            let result = run_back_task(split_func).await;
            let mut response = SplitElfResponse {
                proof_id: request.get_ref().proof_id.clone(),
                computed_request_id: request.get_ref().computed_request_id.clone(),
                total_steps: result.clone().unwrap_or_default().1,
                ..Default::default()
            };
            // True if and only if no error occurs and ELF size > 0
            let result: std::result::Result<(bool, Vec<u8>), String> = match result {
                Ok(cycle) => Ok((cycle.1 > 0 && cycle.0, vec![])),
                Err(e) => Err(e),
            };
            on_done!(result, response);
            let end = Instant::now();
            let elapsed = end.duration_since(start);
            log::info!(
                "[split_elf] {}:{} code:{} elapsed:{} end",
                request.get_ref().proof_id,
                request.get_ref().computed_request_id,
                response.result.as_ref().unwrap().code,
                elapsed.as_secs()
            );
            Ok(Response::new(response))
        })
        .await
    }

    async fn prove(
        &self,
        request: Request<ProveRequest>,
    ) -> tonic::Result<Response<ProveResponse>, Status> {
        metrics::record_metrics("prover::prove", || async {
            log::info!(
                "[prove] {}:{} start",
                request.get_ref().proof_id,
                request.get_ref().computed_request_id,
                //request.get_ref().seg_path,
            );
            let start = Instant::now();

            let prove_context = ProveContext::new(
                request.get_ref().block_no,
                request.get_ref().seg_size,
                &request.get_ref().segment,
                &request.get_ref().receipts_input,
            );

            let pipeline = self.pipeline.clone();
            let prove_func = move || {
                let s_ctx: ProveContext = prove_context;
                pipeline.lock().unwrap().prove_root(&s_ctx)
            };
            let result = run_back_task(prove_func).await;
            let mut response = ProveResponse {
                proof_id: request.get_ref().proof_id.clone(),
                computed_request_id: request.get_ref().computed_request_id.clone(),
                output_receipt: match &result {
                    Ok((_, x)) => x.clone(),
                    _ => vec![],
                },
                ..Default::default()
            };
            on_done!(result, response);
            let end = Instant::now();
            let elapsed = end.duration_since(start);
            log::info!(
                "[prove] {}:{} code:{} elapsed:{} end",
                request.get_ref().proof_id,
                request.get_ref().computed_request_id,
                response.result.as_ref().unwrap().code,
                elapsed.as_secs()
            );
            Ok(Response::new(response))
        })
        .await
    }

    async fn aggregate(
        &self,
        request: Request<AggregateRequest>,
    ) -> tonic::Result<Response<AggregateResponse>, Status> {
        metrics::record_metrics("prover::aggregate", || async {
            log::info!(
                "[aggregate] {}:{} {}+{} start",
                request.get_ref().proof_id,
                request.get_ref().computed_request_id,
                request
                    .get_ref()
                    .input1
                    .clone()
                    .expect("need input1")
                    .computed_request_id,
                request
                    .get_ref()
                    .input2
                    .clone()
                    .expect("need input2")
                    .computed_request_id,
            );
            let start = Instant::now();
            let input1 = request.get_ref().input1.clone().expect("need input1");
            let input2 = request.get_ref().input2.clone().expect("need input2");
            let agg_context = AggContext::new(
                request.get_ref().seg_size,
                &input1.receipt_input,
                &input2.receipt_input,
                input1.is_agg,
                input2.is_agg,
                request.get_ref().is_final,
            );

            let pipeline = self.pipeline.clone();
            let agg_func = move || {
                let agg_ctx = agg_context;
                pipeline.lock().unwrap().prove_aggregate(&agg_ctx)
            };
            let result = run_back_task(agg_func).await;
            let mut response = AggregateResponse {
                proof_id: request.get_ref().proof_id.clone(),
                computed_request_id: request.get_ref().computed_request_id.clone(),
                agg_receipt: match &result {
                    Ok((_, x)) => x.clone(),
                    _ => vec![],
                },
                ..Default::default()
            };
            on_done!(result, response);
            let end = Instant::now();
            let elapsed = end.duration_since(start);
            log::info!(
                "[aggregate] {}:{} code:{} elapsed:{} end",
                request.get_ref().proof_id,
                request.get_ref().computed_request_id,
                response.result.as_ref().unwrap().code,
                elapsed.as_secs()
            );
            Ok(Response::new(response))
        })
        .await
    }

    async fn snark_proof(
        &self,
        request: Request<SnarkProofRequest>,
    ) -> tonic::Result<Response<SnarkProofResponse>, Status> {
        metrics::record_metrics("prover::snark_proof", || async {
            log::info!(
                "[snark_proof] {}:{} start",
                request.get_ref().proof_id,
                request.get_ref().computed_request_id,
            );
            let start = Instant::now();

            let snark_context = SnarkContext {
                version: request.get_ref().version,
                proof_id: request.get_ref().proof_id.clone(),
                proving_key_path: self.config.get_proving_key_path(request.get_ref().version),
                agg_receipt: request.get_ref().agg_receipt.clone(),
            };

            let pipeline = self.pipeline.clone();
            let snark_func = move || {
                let ctx: SnarkContext = snark_context;
                pipeline.lock().unwrap().prove_snark(&ctx)
            };
            let result = run_back_task(snark_func).await;
            let mut response = SnarkProofResponse {
                proof_id: request.get_ref().proof_id.clone(),
                computed_request_id: request.get_ref().computed_request_id.clone(),
                snark_proof_with_public_inputs: match &result {
                    Ok((_, x)) => x.clone(),
                    _ => vec![],
                },
                ..Default::default()
            };
            on_done!(result, response);
            let end = Instant::now();
            let elapsed = end.duration_since(start);
            log::info!(
                "[snark_proof] {}:{} code:{} elapsed:{} end",
                request.get_ref().proof_id,
                request.get_ref().computed_request_id,
                response.result.as_ref().unwrap().code,
                elapsed.as_secs()
            );
            Ok(Response::new(response))
        })
        .await
    }
}
