use crate::proto::prover_service::v1::{
    prover_service_client::ProverServiceClient, AggregateRequest, GetTaskResultRequest,
    GetTaskResultResponse, ProveRequest, ResultCode, SnarkProofRequest, SplitElfRequest,
};
use common::tls::Config as TlsConfig;
use std::sync::{Arc, Mutex};

use crate::stage::tasks::{
    AggTask, ProveTask, SnarkTask, SplitTask, TASK_STATE_FAILED, TASK_STATE_PROCESSING,
    TASK_STATE_SUCCESS, TASK_STATE_UNPROCESSED, TASK_TIMEOUT,
};
use tonic::Request;

use crate::prover_node::{NodeStatus, ProverNode};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use std::time::Duration;
use tonic::transport::Channel;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum TaskType {
    Split,
    Prove,
    Agg,
    Snark,
}

fn get_nodes(task_type: TaskType) -> Vec<ProverNode> {
    let nodes_data = crate::prover_node::instance().lock().unwrap();

    match task_type {
        TaskType::Snark => nodes_data.get_snark_nodes(),
        TaskType::Prove => {
            let all_nodes = nodes_data.get_nodes();
            let nodes_num = std::env::var("PROVE_NODES_NUM")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(all_nodes.len());
            all_nodes.into_iter().take(nodes_num).collect()
        }
        _ => nodes_data.get_nodes(),
    }
}

async fn get_idle_client(
    tls_config: Option<TlsConfig>,
    task_type: TaskType,
) -> Option<(String, ProverServiceClient<Channel>, Arc<Mutex<NodeStatus>>)> {
    let mut nodes = get_nodes(task_type);
    let mut rng = StdRng::from_entropy();
    nodes.shuffle(&mut rng);

    for mut node in nodes {
        let is_idle = {
            let status = node.status.lock().unwrap();
            *status == NodeStatus::Idle
        };

        if !is_idle {
            continue;
        }

        if let Some(client) = node.is_active(tls_config.clone()).await {
            let mut status = node.status.lock().unwrap();
            *status = NodeStatus::Busy;

            return Some((node.addr.clone(), client, node.status.clone()));
        }
    }
    // tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    None
}

pub fn result_code_to_state(code: i32) -> u32 {
    match ResultCode::from_i32(code) {
        Some(ResultCode::Unspecified) => TASK_STATE_PROCESSING,
        Some(ResultCode::Ok) => TASK_STATE_SUCCESS,
        Some(ResultCode::InternalError) => TASK_STATE_FAILED,
        Some(ResultCode::Busy) => TASK_STATE_UNPROCESSED,
        _ => TASK_STATE_FAILED,
    }
}

pub async fn split(mut split_task: SplitTask, tls_config: Option<TlsConfig>) -> Option<SplitTask> {
    split_task.state = TASK_STATE_UNPROCESSED;
    let client = get_idle_client(tls_config, TaskType::Split).await;
    if let Some((addrs, mut client, node_status)) = client {
        let request = SplitElfRequest {
            proof_id: split_task.proof_id.clone(),
            computed_request_id: split_task.task_id.clone(),
            elf_path: split_task.elf_path.clone(),
            base_dir: split_task.base_dir.clone(),
            seg_path: split_task.seg_path.clone(),
            public_input_path: split_task.public_input_path.clone(),
            private_input_path: split_task.private_input_path.clone(),
            output_path: split_task.output_path.clone(),
            args: split_task.args.clone(),
            block_no: split_task.block_no,
            seg_size: split_task.seg_size,
            receipt_inputs_path: split_task.recepit_inputs_path.clone(),
        };
        log::info!(
            "[split] rpc {} {}:{} start",
            addrs,
            request.proof_id,
            request.computed_request_id
        );
        let mut grpc_request = Request::new(request);
        grpc_request.set_timeout(Duration::from_secs(TASK_TIMEOUT));
        let response = client.split_elf(grpc_request).await;
        let mut status = node_status.lock().unwrap();
        *status = NodeStatus::Idle;
        if let Ok(response) = response {
            if let Some(response_result) = response.get_ref().result.as_ref() {
                split_task.state = result_code_to_state(response_result.code);
                // FIXME: node_info usage?
                split_task.trace.node_info = addrs.clone();
                split_task.total_steps = response.get_ref().total_steps;
                log::info!(
                    "[split] rpc {} {}:{} code:{:?} message:{:?} end",
                    addrs,
                    response.get_ref().proof_id,
                    response.get_ref().computed_request_id,
                    response_result.code,
                    response_result.message,
                );
                return Some(split_task);
            }
        }
    }
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    Some(split_task)
}

pub async fn prove(mut prove_task: ProveTask, tls_config: Option<TlsConfig>) -> Option<ProveTask> {
    prove_task.state = TASK_STATE_UNPROCESSED;
    let client = get_idle_client(tls_config, TaskType::Prove).await;
    if let Some((addrs, mut client, node_status)) = client {
        let request = ProveRequest {
            proof_id: prove_task.program.proof_id.clone(),
            computed_request_id: prove_task.task_id.clone(),
            segment: prove_task.segment.clone(),
            block_no: prove_task.program.block_no,
            seg_size: prove_task.program.seg_size,
            receipts_input: prove_task.program.receipts.clone(),
            index: prove_task.file_no as u32,
            done: prove_task.done,
        };
        log::info!(
            "[prove] rpc {} {}:{}start",
            addrs,
            request.proof_id,
            request.computed_request_id,
        );
        let mut grpc_request = Request::new(request);
        grpc_request.set_timeout(Duration::from_secs(TASK_TIMEOUT));
        let response = client.prove(grpc_request).await;
        let mut status = node_status.lock().unwrap();
        *status = NodeStatus::Idle;
        if let Ok(response) = response {
            if let Some(response_result) = response.get_ref().result.as_ref() {
                prove_task.state = result_code_to_state(response_result.code);
                prove_task.trace.node_info = addrs.clone();
                log::info!(
                    "[prove] rpc {} {}:{} code:{:?} message:{:?} end",
                    addrs,
                    response.get_ref().proof_id,
                    response.get_ref().computed_request_id,
                    response_result.code,
                    response_result.message,
                );
                prove_task.output = response.get_ref().output_receipt.clone();
                return Some(prove_task);
            }
        }
    }
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    Some(prove_task)
}

pub async fn aggregate(mut agg_task: AggTask, tls_config: Option<TlsConfig>) -> Option<AggTask> {
    agg_task.state = TASK_STATE_UNPROCESSED;
    let client = get_idle_client(tls_config, TaskType::Agg).await;
    if let Some((addrs, mut client, node_status)) = client {
        let request = AggregateRequest {
            proof_id: agg_task.proof_id.clone(),
            computed_request_id: agg_task.task_id.clone(),
            block_no: agg_task.block_no,
            seg_size: agg_task.seg_size,
            vk: agg_task.vk.clone(),
            inputs: agg_task.inputs.clone(),
            is_final: agg_task.is_final,
            is_first_shard: agg_task.is_first_shard,
            is_leaf_layer: agg_task.is_leaf_layer,
        };
        log::info!(
            "[aggregate] rpc {} {}:{} {} inputs start",
            addrs,
            request.proof_id,
            request.computed_request_id,
            request.inputs.len()
        );
        let mut grpc_request = Request::new(request);
        grpc_request.set_timeout(Duration::from_secs(TASK_TIMEOUT));
        let response = client.aggregate(grpc_request).await;
        let mut status = node_status.lock().unwrap();
        *status = NodeStatus::Idle;
        if let Ok(response) = response {
            if let Some(response_result) = response.get_ref().result.as_ref() {
                agg_task.state = result_code_to_state(response_result.code);
                agg_task.trace.node_info = addrs.clone();
                log::info!(
                    "[aggregate] rpc {} {}:{} code:{:?} message:{:?} end",
                    addrs,
                    response.get_ref().proof_id,
                    response.get_ref().computed_request_id,
                    response_result.code,
                    response_result.message,
                );
                agg_task.output = response.get_ref().agg_receipt.clone();
                return Some(agg_task);
            }
        }
    }
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    Some(agg_task)
}
pub async fn snark_proof(
    mut snark_task: SnarkTask,
    tls_config: Option<TlsConfig>,
) -> Option<SnarkTask> {
    let client = get_idle_client(tls_config, TaskType::Snark).await;
    if let Some((addrs, mut client, node_status)) = client {
        let request = SnarkProofRequest {
            version: snark_task.version,
            proof_id: snark_task.proof_id.clone(),
            computed_request_id: snark_task.task_id.clone(),
            agg_receipt: snark_task.agg_receipt.clone(),
        };
        log::info!(
            "[snark_proof] rpc {} {}:{} start",
            addrs,
            request.proof_id,
            request.computed_request_id,
        );
        let mut grpc_request = Request::new(request);
        grpc_request.set_timeout(Duration::from_secs(TASK_TIMEOUT));
        let response = client.snark_proof(grpc_request).await;
        let mut status = node_status.lock().unwrap();
        *status = NodeStatus::Idle;
        if let Ok(response) = response {
            if let Some(response_result) = response.get_ref().result.as_ref() {
                if ResultCode::from_i32(response_result.code) == Some(ResultCode::Ok) {
                    log::info!(
                        "[snark_proof] rpc {} {}:{}  code:{:?} message:{:?}",
                        addrs,
                        response.get_ref().proof_id,
                        response.get_ref().computed_request_id,
                        response_result.code,
                        response_result.message,
                    );
                    snark_task.state = TASK_STATE_SUCCESS;
                    snark_task.trace.node_info = addrs;
                    snark_task.output = response.get_ref().snark_proof_with_public_inputs.clone();
                    return Some(snark_task);
                }
            }
        }
        snark_task.state = TASK_STATE_FAILED;
    } else {
        snark_task.state = TASK_STATE_UNPROCESSED;
    }
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    Some(snark_task)
}

#[allow(dead_code)]
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
    grpc_request.set_timeout(Duration::from_secs(TASK_TIMEOUT));
    let response = client.get_task_result(grpc_request).await;
    if let Ok(response) = response {
        if let Some(response_result) = response.get_ref().result.as_ref() {
            return ResultCode::from_i32(response_result.code);
        }
    }
    Some(ResultCode::Unspecified)
}

pub async fn get_task_result(
    client: &mut ProverServiceClient<Channel>,
    proof_id: &str,
    task_id: &str,
) -> Option<GetTaskResultResponse> {
    let request = GetTaskResultRequest {
        proof_id: proof_id.to_owned(),
        computed_request_id: task_id.to_owned(),
    };
    let mut grpc_request = Request::new(request);
    grpc_request.set_timeout(Duration::from_secs(30));
    let response = client.get_task_result(grpc_request).await;
    if let Ok(response) = response {
        return Some(response.into_inner());
    }
    None
}
