use executor::split_context::SplitContext;
use prover::contexts::{AggAllContext, AggContext, ProveContext};
use prover::pipeline::Pipeline;
use prover_service::prover_service_server::ProverService;
use prover_service::{get_status_response, GetStatusRequest, GetStatusResponse};
use prover_service::{AggregateAllRequest, AggregateAllResponse};
use prover_service::{AggregateRequest, AggregateResponse};
use prover_service::{FinalProofRequest, FinalProofResponse};
use prover_service::{GetTaskResultRequest, GetTaskResultResponse, Result};
use prover_service::{ProveRequest, ProveResponse};
use prover_service::{SplitElfRequest, SplitElfResponse};
use std::time::Instant;
use tonic::{Request, Response, Status};

use self::prover_service::ResultCode;

use crate::metrics;
use std::panic;

#[allow(clippy::module_inception)]
pub mod prover_service {
    tonic::include_proto!("prover.v1");
}

async fn run_back_task<F: FnOnce() -> std::result::Result<bool, String> + Send + 'static>(
    callable: F,
) -> std::result::Result<bool, String> {
    let rt = tokio::runtime::Handle::current();
    let (tx, rx) = tokio::sync::oneshot::channel();
    let _ = rt
        .spawn_blocking(move || {
            let result = panic::catch_unwind(panic::AssertUnwindSafe(callable));
            // tx.send(result).unwrap();
            // let result = callable();
            let _ = tx.send(result);
        })
        .await;
    match rx.await.unwrap() {
        Ok(result) => result,
        Err(e) => {
            log::error!("{:#?}", e);
            Err("panic".to_string())
        }
    }
}

#[derive(Debug, Default)]
pub struct ProverServiceSVC {}

macro_rules! on_done {
    ($result:ident, $resp:ident) => {
        match $result {
            Ok(done) => {
                if done {
                    $resp.result = Some(Result {
                        code: (ResultCode::Ok.into()),
                        message: ("SUCCESS".to_string()),
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

            let mut response = prover_service::GetStatusResponse::default();
            let success = Pipeline::new().get_status();
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
            let response = prover_service::GetTaskResultResponse::default();
            Ok(Response::new(response))
        })
        .await
    }

    async fn split_elf(
        &self,
        request: Request<SplitElfRequest>,
    ) -> tonic::Result<Response<SplitElfResponse>, Status> {
        metrics::record_metrics("prover::split_elf", || async {
            log::info!("{:#?}", request);
            let start = Instant::now();

            let split_context = SplitContext::new(
                &request.get_ref().base_dir,
                &request.get_ref().elf_path,
                request.get_ref().block_no,
                request.get_ref().seg_size,
                &request.get_ref().seg_path,
                &request.get_ref().args,
            );
            let split_func = move || {
                let s_ctx: SplitContext = split_context;
                executor::executor::Executor::new().split(&s_ctx)
            };
            let result = run_back_task(split_func).await;
            let mut response = prover_service::SplitElfResponse {
                proof_id: request.get_ref().proof_id.clone(),
                computed_request_id: request.get_ref().computed_request_id.clone(),
                ..Default::default()
            };
            on_done!(result, response);
            let end = Instant::now();
            let elapsed = end.duration_since(start);
            log::info!(
                "split {} elapsed time: {:?} secs",
                request.get_ref().computed_request_id,
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
            log::info!("{:#?}", request);
            let start = Instant::now();

            let prove_context = ProveContext::new(
                &request.get_ref().base_dir,
                request.get_ref().block_no,
                request.get_ref().seg_size,
                &request.get_ref().seg_path,
                &request.get_ref().proof_path,
                &request.get_ref().pub_value_path,
            );

            let prove_func = move || {
                let s_ctx: ProveContext = prove_context;
                Pipeline::new().prove_root(&s_ctx)
            };
            let result = run_back_task(prove_func).await;
            let mut response = prover_service::ProveResponse {
                proof_id: request.get_ref().proof_id.clone(),
                computed_request_id: request.get_ref().computed_request_id.clone(),
                ..Default::default()
            };
            on_done!(result, response);
            let end = Instant::now();
            let elapsed = end.duration_since(start);
            log::info!(
                "prove {} elapsed time: {:?} secs",
                request.get_ref().computed_request_id,
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
            log::info!("{:#?}", request);
            let start = Instant::now();
            let input1 = request.get_ref().input1.clone().expect("need input1");
            let input2 = request.get_ref().input2.clone().expect("need input2");
            let agg_context = AggContext::new(
                &request.get_ref().base_dir,
                request.get_ref().block_no,
                request.get_ref().seg_size,
                &input1.proof_path,
                &input2.proof_path,
                &input1.pub_value_path,
                &input2.pub_value_path,
                input1.is_agg,
                input2.is_agg,
                request.get_ref().is_final,
                &request.get_ref().agg_proof_path,
                &request.get_ref().agg_pub_value_path,
                &request.get_ref().output_dir,
            );

            let agg_func = move || {
                let agg_ctx = agg_context;
                Pipeline::new().prove_aggregate(&agg_ctx)
            };
            let result = run_back_task(agg_func).await;
            let mut response = prover_service::AggregateResponse {
                proof_id: request.get_ref().proof_id.clone(),
                computed_request_id: request.get_ref().computed_request_id.clone(),
                ..Default::default()
            };
            on_done!(result, response);
            let end = Instant::now();
            let elapsed = end.duration_since(start);
            log::info!(
                "aggregate {} elapsed time: {:?} secs",
                request.get_ref().computed_request_id,
                elapsed.as_secs()
            );
            Ok(Response::new(response))
        })
        .await
    }

    async fn aggregate_all(
        &self,
        request: Request<AggregateAllRequest>,
    ) -> tonic::Result<Response<AggregateAllResponse>, Status> {
        metrics::record_metrics("prover::aggregate", || async {
            log::info!("{:#?}", request);
            let start = Instant::now();
            let final_context = AggAllContext::new(
                &request.get_ref().base_dir,
                request.get_ref().block_no,
                request.get_ref().seg_size,
                request.get_ref().proof_num,
                &request.get_ref().proof_dir,
                &request.get_ref().pub_value_dir,
                &request.get_ref().output_dir,
            );

            let agg_all_func = move || {
                let s_ctx: AggAllContext = final_context;
                Pipeline::new().prove_aggregate_all(&s_ctx)
            };
            let result = run_back_task(agg_all_func).await;
            let mut response = prover_service::AggregateAllResponse {
                proof_id: request.get_ref().proof_id.clone(),
                computed_request_id: request.get_ref().computed_request_id.clone(),
                ..Default::default()
            };
            on_done!(result, response);
            let end = Instant::now();
            let elapsed = end.duration_since(start);
            log::info!(
                "aggregate_all {} elapsed time: {:?} secs",
                request.get_ref().computed_request_id,
                elapsed.as_secs()
            );
            Ok(Response::new(response))
        })
        .await
    }

    async fn final_proof(
        &self,
        _request: Request<FinalProofRequest>,
    ) -> tonic::Result<Response<FinalProofResponse>, Status> {
        metrics::record_metrics("prover::final_proof", || async {
            // log::info!("{:#?}", request);
            let response = prover_service::FinalProofResponse::default();
            Ok(Response::new(response))
        })
        .await
    }
}
