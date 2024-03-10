use std::borrow::BorrowMut;
use std::result;

use prover_service::prover_service_server::ProverService;
use prover_service::{Result};
use prover_service::{GetStatusRequest, GetStatusResponse};
use prover_service::{SplitElfRequest, SplitElfResponse};
use prover_service::{ProveRequest, ProveResponse};
use prover_service::{AggregateRequest, AggregateResponse};
use prover_service::{AggregateAllRequest, AggregateAllResponse};
use prover::contexts::{SplitContext, ProveContext, AggContext, FinalContext};

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
        println!("{:#?}", request);

        let status= Pipeline::new().get_status();
        match status {
            Ok(mesage) => {
                print!("SUCCESS")
            }
            _ => {
                print!("FAILED")
            }
        };


        let mut response = prover_service::GetStatusResponse::default();
        response.last_computed_request_id = String::from("");
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
        let result = Pipeline::new().split_prove(&split_context);
        let mut response = prover_service::SplitElfResponse::default();
        response.proof_id = request.get_ref().proof_id.clone();
        response.computed_request_id = request.get_ref().computed_request_id.clone();
        match result {
            Ok(message) => {
                response.result = Some(Result { code: (ResultCode::ResultOk.into()), message: (message) });
            }
            Err(e) => {
                let errmsg = format!("{:#?}", e);
                response.result = Some(Result { code: (ResultCode::ResultError.into()), message: (errmsg) });
            }
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
        let result = Pipeline::new().root_prove(&prove_context);
        let mut response = prover_service::ProveResponse::default();
        response.proof_id = request.get_ref().proof_id.clone();
        response.computed_request_id = request.get_ref().computed_request_id.clone();
        match result {
            Ok(message) => {
                response.result = Some(Result { code: (ResultCode::ResultOk.into()), message: (message) });
            }
            Err(e) => {
                let errmsg = format!("{:#?}", e);
                response.result = Some(Result { code: (ResultCode::ResultError.into()), message: (errmsg) });
            }
        }
        Ok(Response::new(response))
    }

    async fn aggregate(
        &self, 
        request: Request<AggregateRequest>
    ) -> tonic::Result<Response<AggregateResponse>, Status> {
        println!("{:#?}", request);
        let mut response = prover_service::AggregateResponse::default();
        response.computed_request_id = request.get_ref().computed_request_id.clone();
        Ok(Response::new(response))
    }

    async fn aggregate_all(
        &self, 
        request: Request<AggregateAllRequest>
    ) -> tonic::Result<Response<AggregateAllResponse>, Status> {
        println!("{:#?}", request);
        let mut response = prover_service::AggregateAllResponse::default();
        response.computed_request_id = request.get_ref().computed_request_id.clone();
        Ok(Response::new(response))
    }

}
