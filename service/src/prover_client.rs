use std::borrow::BorrowMut;
use std::result;

use prover_service::prover_service_client::ProverServiceClient;
use prover_service::{Result};
use prover_service::{get_status_response, GetStatusRequest, GetStatusResponse};
use prover_service::{SplitElfRequest, SplitElfResponse};
use prover_service::{ProveRequest, ProveResponse};
use prover_service::{AggregateRequest, AggregateResponse};
use prover_service::{AggregateAllRequest, AggregateAllResponse};

use prover::pipeline::{self,Pipeline};

use tonic::{client, Request, Response, Status};
use stage::tasks::{AggAllTask, FinalTask, ProveTask, SplitTask, TASK_STATE_FAILED, TASK_STATE_SUCCESS, TASK_STATE_UNPROCESSED};

use self::prover_service::ResultCode;
use tonic::transport::{Uri}; 
use tonic::transport::Channel;
use std::net::ToSocketAddrs;
use crate::prover_node::ProverNode;
use crate::prover_node::ProverNodes;
use std::time::Duration;

pub mod prover_service {
    tonic::include_proto!("prover.v1");
}

pub async fn get_idle_client() -> Option<ProverServiceClient<Channel>> {
    let nodes: Vec<ProverNode>;
    {
        let nodes_lock = crate::prover_node::instance();
        let nodes_data = nodes_lock.lock().unwrap();
        nodes = nodes_data.get_nodes();
    }
    for node in nodes {
        let client = is_active(&node.addr).await;
        if let Some(client) = client {
            return Some(client);
        }
    }
    return None;
}

pub async fn is_active(addr: &String) -> Option<ProverServiceClient<Channel>> {
    let uri = format!("grpc://{}", addr).parse::<Uri>().unwrap();
    let endpoint = tonic::transport::Channel::builder(uri);
    let client = ProverServiceClient::connect(endpoint).await;
    if let Ok(mut client) = client {
        let request = GetStatusRequest {};
        let response = client.get_status(Request::new(request)).await;
        if let Ok(response) = response {
            let status = response.get_ref().status;
            if get_status_response::Status::from_i32(status) == Some(get_status_response::Status::Idle) {
                return Some(client);
            }
        }
    }
    return None;
}

pub async fn split(mut split_task: SplitTask) -> Option<SplitTask> {
    let client = get_idle_client().await;  
    if let Some(mut client) = client {
        let mut request = SplitElfRequest::default();
        request.proof_id = split_task.proof_id.clone();
        request.computed_request_id = split_task.task_id.clone();
        request.base_dir = split_task.base_dir.clone();
        request.elf_path = split_task.elf_path.clone();
        request.seg_path = split_task.seg_path.clone();
        request.block_no = split_task.block_no;
        request.seg_size = split_task.seg_size;
        print!("split request {:#?}", request);
        let mut grpc_request = Request::new(request);
        grpc_request.set_timeout(Duration::from_secs(300));
        let response = client.split_elf(grpc_request).await;
        if let Ok(response) = response {
            if let Some(response_result) = response.get_ref().result.as_ref() {
                print!("split response {:#?}", response);
                if ResultCode::from_i32(response_result.code) == Some(ResultCode::ResultOk) {
                    split_task.state = TASK_STATE_SUCCESS;
                    return Some(split_task);
                }
            }
        }
        split_task.state = TASK_STATE_FAILED;
    } else {
        split_task.state = TASK_STATE_UNPROCESSED;
    }
    Some(split_task)
}

pub async fn prove(mut prove_task: ProveTask) -> Option<ProveTask> {
    let client = get_idle_client().await;  
    if let Some(mut client) = client {
        let mut request = ProveRequest::default();
        request.proof_id = prove_task.proof_id.clone();
        request.computed_request_id = prove_task.task_id.clone();
        request.base_dir = prove_task.base_dir.clone();
        request.seg_path = prove_task.seg_path.clone();
        request.block_no = prove_task.block_no;
        request.seg_size = prove_task.seg_size;
        request.proof_path = prove_task.prove_path.clone();
        request.pub_value_path = prove_task.pub_value_path.clone();
        print!("prove request {:#?}", request);
        let mut grpc_request = Request::new(request);
        grpc_request.set_timeout(Duration::from_secs(3000));
        let response = client.prove(grpc_request).await;
        if let Ok(response) = response {
            if let Some(response_result) = response.get_ref().result.as_ref() {
                print!("prove response {:#?}", response);
                if ResultCode::from_i32(response_result.code) == Some(ResultCode::ResultOk) {
                    prove_task.state = TASK_STATE_SUCCESS;
                    return Some(prove_task);
                }
            }
        }
        prove_task.state = TASK_STATE_FAILED;
    } else {
        prove_task.state = TASK_STATE_UNPROCESSED;
    }
    Some(prove_task)
}

pub async fn aggregate_all(mut agg_all_task: AggAllTask) -> Option<AggAllTask> {
    let client = get_idle_client().await;  
    if let Some(mut client) = client {
        let mut request = AggregateAllRequest::default();
        request.proof_id = agg_all_task.proof_id.clone();
        request.computed_request_id = agg_all_task.task_id.clone();
        request.base_dir = agg_all_task.base_dir.clone();
        request.block_no = agg_all_task.block_no;
        request.seg_size = agg_all_task.seg_size;
        request.proof_num = agg_all_task.proof_num;
        request.proof_dir = agg_all_task.proof_dir.clone();
        request.pub_value_dir = agg_all_task.pub_value_dir.clone();
        request.output_dir = agg_all_task.output_dir.clone();

        print!("aggregate request {:#?}", request);
        let mut grpc_request = Request::new(request);
        grpc_request.set_timeout(Duration::from_secs(3000));
        let response = client.aggregate_all(grpc_request).await;
        if let Ok(response) = response {
            if let Some(response_result) = response.get_ref().result.as_ref() {
                print!("aggregate response {:#?}", response);
                if ResultCode::from_i32(response_result.code) == Some(ResultCode::ResultOk) {
                    agg_all_task.state = TASK_STATE_SUCCESS;
                    return Some(agg_all_task);
                }
            }
        }
        agg_all_task.state = TASK_STATE_FAILED;
    } else {
        agg_all_task.state = TASK_STATE_UNPROCESSED;
    }
    Some(agg_all_task)
}

pub async fn final_proof(mut final_task: FinalTask) -> Option<FinalTask> {
    // let client = get_idle_client().await;  
    // if let Some(mut client) = client {
    //     let mut request = AggregateAllRequest::default();
    //     request.proof_id = final_task.proof_id.clone();
    //     request.computed_request_id = final_task.task_id.clone();
    //     request.base_dir = final_task.base_dir.clone();
    //     request.block_no = final_task.block_no;
    //     request.seg_size = final_task.seg_size;
    //     request.pub_value_dir = final_task.pub_value_path.clone();
    //     request.output_dir = final_task.out_path.clone();

    //     print!("aggregate_all request {:#?}", request);
    //     let mut grpc_request = Request::new(request);
    //     grpc_request.set_timeout(Duration::from_secs(3000));
    //     let response = client.aggregate_all(grpc_request).await;
    //     if let Ok(response) = response {
    //         if let Some(response_result) = response.get_ref().result.as_ref() {
    //             print!("aggregate_all response {:#?}", response);
    //             if ResultCode::from_i32(response_result.code) == Some(ResultCode::ResultOk) {
    //                 final_task.state = TASK_STATE_SUCCESS;
    //                 return Some(final_task);
    //             }
    //         }
    //     }
    //     final_task.state = TASK_STATE_FAILED;
    // } else {
    //     final_task.state = TASK_STATE_UNPROCESSED;
    // }
    Some(final_task)
}