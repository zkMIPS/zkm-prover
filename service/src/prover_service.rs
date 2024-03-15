use std::borrow::BorrowMut;
use std::result;

use prover_service::prover_service_server::ProverService;
use prover_service::{Result};
use prover_service::{get_status_response, GetStatusRequest, GetStatusResponse};
use prover_service::{GetTaskResultRequest, GetTaskResultResponse};
use prover_service::{SplitElfRequest, SplitElfResponse};
use prover_service::{ProveRequest, ProveResponse};
use prover_service::{AggregateRequest, AggregateResponse};
use prover_service::{AggregateAllRequest, AggregateAllResponse};
use prover_service::{FinalProofRequest, FinalProofResponse};
use prover::contexts::{agg_context, AggContext, AggAllContext, ProveContext, SplitContext};

use prover::pipeline::{self,Pipeline};

use tonic::{Request, Response, Status};

use self::prover_service::ResultCode;
pub mod prover_service {
    tonic::include_proto!("prover.v1");
}

#[derive(Debug,Default)]
pub struct ProverServiceSVC{
}

#[tonic::async_trait]
impl ProverService for ProverServiceSVC {
    async fn get_status(
        &self,
        request: Request<GetStatusRequest>
    ) -> tonic::Result<Response<GetStatusResponse>, Status> {
        // println!("{:#?}", request);

        let mut response = prover_service::GetStatusResponse::default();
        let success= Pipeline::new().get_status();
        if success {
            response.status = get_status_response::Status::Idle.into();
        } else {
            response.status = get_status_response::Status::Computing.into();
        }
        Ok(Response::new(response))
    }

    async fn get_task_result(
        &self,
        request: Request<GetTaskResultRequest>
    ) -> tonic::Result<Response<GetTaskResultResponse>, Status> {
        // println!("{:#?}", request);
        let response = prover_service::GetTaskResultResponse::default();
        Ok(Response::new(response))
    }

    
    async fn split_elf(
        &self, 
        request: Request<SplitElfRequest>
    ) -> tonic::Result<Response<SplitElfResponse>, Status> {
        println!("{:#?}", request);

        let split_context = SplitContext::new(
            &request.get_ref().base_dir,
            &request.get_ref().elf_path, 
            request.get_ref().block_no, 
            request.get_ref().seg_size, 
            &request.get_ref().seg_path); 
        let success = Pipeline::new().split_prove(&split_context);
        let mut response = prover_service::SplitElfResponse::default();
        response.proof_id = request.get_ref().proof_id.clone();
        response.computed_request_id = request.get_ref().computed_request_id.clone();
        if success {
            response.result = Some(Result { code: (ResultCode::ResultOk.into()), message: ("SUCCESS".to_string()) });
        } else {
            response.result = Some(Result { code: (ResultCode::ResultError.into()), message: ("FAILED".to_string()) });
        }
        Ok(Response::new(response))
    }

    async fn prove(
        &self, 
        request: Request<ProveRequest>
    ) -> tonic::Result<Response<ProveResponse>, Status> {
        println!("{:#?}", request);

        let prove_context = ProveContext::new(
            &request.get_ref().base_dir, 
            request.get_ref().block_no, 
            request.get_ref().seg_size, 
            &request.get_ref().seg_path,
            &request.get_ref().proof_path,
            &request.get_ref().pub_value_path); 
        let success = Pipeline::new().root_prove(&prove_context);
        let mut response = prover_service::ProveResponse::default();
        response.proof_id = request.get_ref().proof_id.clone();
        response.computed_request_id = request.get_ref().computed_request_id.clone();
        if success {
            response.result = Some(Result { code: (ResultCode::ResultOk.into()), message: ("SUCCESS".to_string()) });
        } else {
            response.result = Some(Result { code: (ResultCode::ResultError.into()), message: ("FAILED".to_string()) });
        }
        Ok(Response::new(response))
    }

    async fn aggregate(
        &self, 
        request: Request<AggregateRequest>
    ) -> tonic::Result<Response<AggregateResponse>, Status> {
        println!("{:#?}", request);
        let agg_context = AggContext::new(
            &request.get_ref().base_dir,
            request.get_ref().block_no, 
            request.get_ref().seg_size, 
            &request.get_ref().proof_path1, 
            &request.get_ref().proof_path2,
            &request.get_ref().pub_value_path1, 
            &request.get_ref().pub_value_path2,
            request.get_ref().is_agg_1, 
            request.get_ref().is_agg_2, 
            &request.get_ref().agg_proof_path, 
            &request.get_ref().agg_pub_value_path);

        let success = Pipeline::new().aggregate_prove(&agg_context);
        let mut response = prover_service::AggregateResponse::default();
        response.proof_id = request.get_ref().proof_id.clone();
        response.computed_request_id = request.get_ref().computed_request_id.clone();
        if success {
            response.result = Some(Result { code: (ResultCode::ResultOk.into()), message: ("SUCCESS".to_string()) });
        } else {
            response.result = Some(Result { code: (ResultCode::ResultError.into()), message: ("FAILED".to_string()) });
        }
        Ok(Response::new(response))
    }

    async fn aggregate_all(
        &self, 
        request: Request<AggregateAllRequest>
    ) -> tonic::Result<Response<AggregateAllResponse>, Status> {
        println!("{:#?}", request);
        let final_context = AggAllContext::new(
            &request.get_ref().base_dir,
            request.get_ref().block_no, 
            request.get_ref().seg_size, 
            request.get_ref().proof_num,
            &request.get_ref().proof_dir, 
            &request.get_ref().pub_value_dir, 
            &request.get_ref().output_dir);
        
            let success = Pipeline::new().aggregate_all_prove(&final_context);
            let mut response = prover_service::AggregateAllResponse::default();
            response.proof_id = request.get_ref().proof_id.clone();
            response.computed_request_id = request.get_ref().computed_request_id.clone();
            if success {
                response.result = Some(Result { code: (ResultCode::ResultOk.into()), message: ("SUCCESS".to_string()) });
            } else {
                response.result = Some(Result { code: (ResultCode::ResultError.into()), message: ("FAILED".to_string()) });
            }
            Ok(Response::new(response))
    }

    async fn final_proof(
        &self,
        request: Request<FinalProofRequest>
    ) -> tonic::Result<Response<FinalProofResponse>, Status> {
        // println!("{:#?}", request);
        let response = prover_service::FinalProofResponse::default();
        Ok(Response::new(response))
    }
}
