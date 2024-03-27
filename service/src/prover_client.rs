use prover_service::prover_service_client::ProverServiceClient;
use prover_service::AggregateAllRequest;
use prover_service::FinalProofRequest;
use prover_service::GetTaskResultRequest;
use prover_service::ProveRequest;
use prover_service::SplitElfRequest;
use prover_service::{get_status_response, GetStatusRequest};

use stage::tasks::{
    AggAllTask, FinalTask, ProveTask, SplitTask, TASK_STATE_FAILED, TASK_STATE_SUCCESS,
    TASK_STATE_UNPROCESSED, TASK_TIMEOUT,
};
use tonic::Request;

use self::prover_service::ResultCode;
use crate::prover_node::ProverNode;
use std::time::Duration;
use tonic::transport::Channel;
use tonic::transport::Uri;

pub mod prover_service {
    tonic::include_proto!("prover.v1");
}

pub fn get_nodes() -> Vec<ProverNode> {
    let nodes_lock = crate::prover_node::instance();
    let nodes_data = nodes_lock.lock().unwrap();
    nodes_data.get_nodes()
}

pub async fn get_idle_client() -> Option<ProverServiceClient<Channel>> {
    let nodes: Vec<ProverNode> = get_nodes();
    for node in nodes {
        let client = is_active(&node.addr).await;
        if let Some(client) = client {
            return Some(client);
        }
    }
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    None
}

pub fn get_snark_nodes() -> Vec<ProverNode> {
    let nodes_lock = crate::prover_node::instance();
    let nodes_data = nodes_lock.lock().unwrap();
    nodes_data.get_snark_nodes()
}

pub async fn get_snark_client() -> Option<ProverServiceClient<Channel>> {
    let nodes: Vec<ProverNode> = get_snark_nodes();
    for node in nodes {
        let client = is_active(&node.addr).await;
        if let Some(client) = client {
            return Some(client);
        }
    }
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    None
}

pub async fn is_active(addr: &String) -> Option<ProverServiceClient<Channel>> {
    let uri = format!("grpc://{}", addr).parse::<Uri>().unwrap();
    let endpoint = tonic::transport::Channel::builder(uri)
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(TASK_TIMEOUT))
        .concurrency_limit(256);
    let client = ProverServiceClient::connect(endpoint).await;
    if let Ok(mut client) = client {
        let request = GetStatusRequest {};
        let response = client.get_status(Request::new(request)).await;
        if let Ok(response) = response {
            let status = response.get_ref().status;
            if get_status_response::Status::from_i32(status)
                == Some(get_status_response::Status::Idle)
                || get_status_response::Status::from_i32(status)
                    == Some(get_status_response::Status::Unspecified)
            {
                return Some(client);
            }
        }
    }
    None
}

pub async fn split(mut split_task: SplitTask) -> Option<SplitTask> {
    let client = get_idle_client().await;
    if let Some(mut client) = client {
        let request = SplitElfRequest {
            chain_id: 0,
            timestamp: 0,
            proof_id: split_task.proof_id.clone(),
            computed_request_id: split_task.task_id.clone(),
            base_dir: split_task.base_dir.clone(),
            elf_path: split_task.elf_path.clone(),
            seg_path: split_task.seg_path.clone(),
            block_no: split_task.block_no,
            seg_size: split_task.seg_size,
        };
        log::info!("split request {:#?}", request);
        let mut grpc_request = Request::new(request);
        grpc_request.set_timeout(Duration::from_secs(300));
        let response = client.split_elf(grpc_request).await;
        if let Ok(response) = response {
            if let Some(response_result) = response.get_ref().result.as_ref() {
                log::info!("split response {:#?}", response);
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
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    Some(split_task)
}

pub async fn prove(mut prove_task: ProveTask) -> Option<ProveTask> {
    let client = get_idle_client().await;
    if let Some(mut client) = client {
        let request = ProveRequest {
            chain_id: 0,
            timestamp: 0,
            proof_id: prove_task.proof_id.clone(),
            computed_request_id: prove_task.task_id.clone(),
            base_dir: prove_task.base_dir.clone(),
            seg_path: prove_task.seg_path.clone(),
            block_no: prove_task.block_no,
            seg_size: prove_task.seg_size,
            proof_path: prove_task.prove_path.clone(),
            pub_value_path: prove_task.pub_value_path.clone(),
        };
        log::info!("prove request {:#?}", request);
        let mut grpc_request = Request::new(request);
        grpc_request.set_timeout(Duration::from_secs(3000));
        let response = client.prove(grpc_request).await;
        if let Ok(response) = response {
            if let Some(response_result) = response.get_ref().result.as_ref() {
                log::info!("prove response {:#?}", response);
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
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    Some(prove_task)
}

pub async fn aggregate_all(mut agg_all_task: AggAllTask) -> Option<AggAllTask> {
    let client = get_idle_client().await;
    if let Some(mut client) = client {
        let request = AggregateAllRequest {
            chain_id: 0,
            timestamp: 0,
            proof_id: agg_all_task.proof_id.clone(),
            computed_request_id: agg_all_task.task_id.clone(),
            base_dir: agg_all_task.base_dir.clone(),
            seg_path: agg_all_task.base_dir.clone(),
            block_no: agg_all_task.block_no,
            seg_size: agg_all_task.seg_size,
            proof_num: agg_all_task.proof_num,
            proof_dir: agg_all_task.proof_dir.clone(),
            pub_value_dir: agg_all_task.pub_value_dir.clone(),
            output_dir: agg_all_task.output_dir.clone(),
        };
        log::info!("aggregate request {:#?}", request);
        let mut grpc_request = Request::new(request);
        grpc_request.set_timeout(Duration::from_secs(3000));
        let response = client.aggregate_all(grpc_request).await;
        if let Ok(response) = response {
            if let Some(response_result) = response.get_ref().result.as_ref() {
                log::info!("aggregate response {:#?}", response);
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
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    Some(agg_all_task)
}

pub async fn final_proof(mut final_task: FinalTask) -> Option<FinalTask> {
    let client = get_snark_client().await;
    if let Some(mut client) = client {
        let request = FinalProofRequest {
            chain_id: 0,
            timestamp: 0,
            proof_id: final_task.proof_id.clone(),
            computed_request_id: final_task.task_id.clone(),
            input_dir: final_task.input_dir.clone(),
            output_path: final_task.output_path.clone(),
        };
        log::info!("final_proof request {:#?}", request);
        let mut grpc_request = Request::new(request);
        grpc_request.set_timeout(Duration::from_secs(3000));
        let response = client.final_proof(grpc_request).await;
        if let Ok(response) = response {
            if let Some(response_result) = response.get_ref().result.as_ref() {
                log::info!("final_proof response {:#?}", response);
                if ResultCode::from_i32(response_result.code) == Some(ResultCode::ResultOk) {
                    let mut loop_count = 0;
                    loop {
                        let task_result =
                            get_task_status(&mut client, &final_task.proof_id, &final_task.task_id)
                                .await;
                        if let Some(task_result) = task_result {
                            if task_result == ResultCode::ResultOk {
                                final_task.state = TASK_STATE_SUCCESS;
                                return Some(final_task);
                            }
                        }
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        loop_count += 1;
                        if loop_count > TASK_TIMEOUT {
                            break;
                        }
                    }
                }
            }
        }
        final_task.state = TASK_STATE_FAILED;
    } else {
        final_task.state = TASK_STATE_UNPROCESSED;
    }
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    Some(final_task)
}

pub async fn get_task_status(
    client: &mut ProverServiceClient<Channel>,
    proof_id: &str,
    task_id: &str,
) -> Option<ResultCode> {
    let request = GetTaskResultRequest {
        proof_id: proof_id.to_owned(),
        computed_request_id: task_id.to_owned(),
    };
    let mut grpc_request = Request::new(request);
    grpc_request.set_timeout(Duration::from_secs(30));
    let response = client.get_task_result(grpc_request).await;
    if let Ok(response) = response {
        if let Some(response_result) = response.get_ref().result.as_ref() {
            return ResultCode::from_i32(response_result.code);
        }
    }
    Some(ResultCode::ResultUnspecified)
}
