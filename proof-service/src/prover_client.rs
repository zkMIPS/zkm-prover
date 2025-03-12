use crate::proto::includes::v1::AggregateInput;
use crate::proto::prover_service::v1::{
    prover_service_client::ProverServiceClient, AggregateAllRequest, AggregateRequest,
    GetTaskResultRequest, GetTaskResultResponse, ProveRequest, ResultCode, SnarkProofRequest,
    SplitElfRequest,
};
use common::file;
use common::tls::Config as TlsConfig;

use crate::stage::tasks::{
    AggAllTask, AggTask, ProveTask, SnarkTask, SplitTask, TASK_STATE_FAILED, TASK_STATE_PROCESSING,
    TASK_STATE_SUCCESS, TASK_STATE_UNPROCESSED, TASK_TIMEOUT,
};
use tonic::Request;

use crate::prover_node::ProverNode;
use std::time::Duration;
use tonic::transport::Channel;

pub fn get_nodes() -> Vec<ProverNode> {
    let nodes_lock = crate::prover_node::instance();
    let nodes_data = nodes_lock.lock().unwrap();
    nodes_data.get_nodes()
}

pub async fn get_idle_client(
    tls_config: Option<TlsConfig>,
) -> Option<(String, ProverServiceClient<Channel>)> {
    let nodes: Vec<ProverNode> = get_nodes();
    for mut node in nodes {
        let client = node.is_active(tls_config.clone()).await;
        if let Some(client) = client {
            return Some((node.addr.clone(), client));
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
    let client = get_idle_client(tls_config).await;
    if let Some((addrs, mut client)) = client {
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
            "[split] rpc {}:{} start",
            request.proof_id,
            request.computed_request_id
        );
        let mut grpc_request = Request::new(request);
        grpc_request.set_timeout(Duration::from_secs(TASK_TIMEOUT));
        let response = client.split_elf(grpc_request).await;
        if let Ok(response) = response {
            if let Some(response_result) = response.get_ref().result.as_ref() {
                split_task.state = result_code_to_state(response_result.code);
                // FIXME: node_info usage?
                split_task.trace.node_info = addrs;
                split_task.total_steps = response.get_ref().total_steps;
                log::info!(
                    "[split] rpc {}:{} code:{:?} message:{:?} end",
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
    let client = get_idle_client(tls_config).await;
    if let Some((addrs, mut client)) = client {
        let request = ProveRequest {
            proof_id: prove_task.program.proof_id.clone(),
            computed_request_id: prove_task.task_id.clone(),
            segment: prove_task.segment.clone(),
            block_no: prove_task.program.block_no,
            seg_size: prove_task.program.seg_size,
            receipts_input: prove_task.program.receipts.clone(),
        };
        log::info!(
            "[prove] rpc {}:{}start",
            request.proof_id,
            request.computed_request_id,
        );
        let mut grpc_request = Request::new(request);
        grpc_request.set_timeout(Duration::from_secs(TASK_TIMEOUT));
        let response = client.prove(grpc_request).await;
        if let Ok(response) = response {
            if let Some(response_result) = response.get_ref().result.as_ref() {
                prove_task.state = result_code_to_state(response_result.code);
                prove_task.trace.node_info = addrs;
                log::info!(
                    "[prove] rpc {}:{} code:{:?} message:{:?} end",
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
    let client = get_idle_client(tls_config).await;
    if let Some((addrs, mut client)) = client {
        let request = AggregateRequest {
            proof_id: agg_task.proof_id.clone(),
            computed_request_id: agg_task.task_id.clone(),
            block_no: agg_task.block_no,
            seg_size: agg_task.seg_size,
            input1: Some(agg_task.input1.clone()),
            input2: Some(agg_task.input2.clone()),
            is_final: agg_task.is_final,
        };
        log::info!(
            "[aggregate] rpc {}:{} {}+{} start",
            request.proof_id,
            request.computed_request_id,
            request
                .input1
                .clone()
                .expect("need input1")
                .computed_request_id,
            request
                .input2
                .clone()
                .expect("need input2")
                .computed_request_id,
        );
        let mut grpc_request = Request::new(request);
        grpc_request.set_timeout(Duration::from_secs(TASK_TIMEOUT));
        let response = client.aggregate(grpc_request).await;
        if let Ok(response) = response {
            if let Some(response_result) = response.get_ref().result.as_ref() {
                agg_task.state = result_code_to_state(response_result.code);
                agg_task.trace.node_info = addrs;
                log::info!(
                    "[aggregate] rpc {}:{} code:{:?} message:{:?} end",
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

pub async fn aggregate_all(
    mut agg_all_task: AggAllTask,
    tls_config: Option<TlsConfig>,
) -> Option<AggAllTask> {
    agg_all_task.state = TASK_STATE_UNPROCESSED;
    let client = get_idle_client(tls_config).await;
    if let Some((addrs, mut client)) = client {
        let request = AggregateAllRequest {
            proof_id: agg_all_task.proof_id.clone(),
            computed_request_id: agg_all_task.task_id.clone(),
            seg_size: agg_all_task.seg_size,
            block_no: agg_all_task.block_no,
            segment: agg_all_task.segment.clone(),
            proof_num: agg_all_task.proof_num,
            receipt_dir: agg_all_task.receipt_dir.clone(),
            output_dir: agg_all_task.output_dir.clone(),
        };
        log::info!(
            "[aggregate_all] rpc {}:{} start",
            request.proof_id,
            request.computed_request_id
        );
        let mut grpc_request = Request::new(request);
        grpc_request.set_timeout(Duration::from_secs(TASK_TIMEOUT));
        let response = client.aggregate_all(grpc_request).await;
        if let Ok(response) = response {
            if let Some(response_result) = response.get_ref().result.as_ref() {
                agg_all_task.state = result_code_to_state(response_result.code);
                agg_all_task.trace.node_info = addrs;
                log::info!(
                    "[aggregate_all] rpc {}:{}  code:{:?} message:{:?}",
                    response.get_ref().proof_id,
                    response.get_ref().computed_request_id,
                    response_result.code,
                    response_result.message,
                );
                return Some(agg_all_task);
            }
        }
    }
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    Some(agg_all_task)
}

pub async fn snark_proof(
    mut snark_task: SnarkTask,
    tls_config: Option<TlsConfig>,
) -> Option<SnarkTask> {
    let client = get_idle_client(tls_config).await;
    if let Some((addrs, mut client)) = client {
        let request = SnarkProofRequest {
            version: snark_task.version,
            proof_id: snark_task.proof_id.clone(),
            computed_request_id: snark_task.task_id.clone(),
            agg_receipt: snark_task.agg_receipt.clone(),
        };
        log::info!(
            "[snark_proof] rpc {}:{} start",
            request.proof_id,
            request.computed_request_id,
        );
        let mut grpc_request = Request::new(request);
        grpc_request.set_timeout(Duration::from_secs(TASK_TIMEOUT));
        let response = client.snark_proof(grpc_request).await;
        if let Ok(response) = response {
            if let Some(response_result) = response.get_ref().result.as_ref() {
                if ResultCode::from_i32(response_result.code) == Some(ResultCode::Ok) {
                    let mut loop_count = 0;
                    loop {
                        let task_result =
                            get_task_result(&mut client, &snark_task.proof_id, &snark_task.task_id)
                                .await;
                        if let Some(task_result) = task_result {
                            if let Some(result) = task_result.result {
                                if let Some(code) = ResultCode::from_i32(result.code) {
                                    if code == ResultCode::Ok {
                                        log::info!(
                                            "[snark_proof] rpc {}:{}  code:{:?} message:{:?}",
                                            response.get_ref().proof_id,
                                            response.get_ref().computed_request_id,
                                            response_result.code,
                                            response_result.message,
                                        );
                                        log::debug!("[snark_proof] rpc {:#?} end", result);
                                        let _ = file::new(&snark_task.output_path)
                                            .write(result.message.as_bytes())
                                            .unwrap();
                                        snark_task.state = TASK_STATE_SUCCESS;
                                        snark_task.trace.node_info = addrs;
                                        snark_task.output = response
                                            .get_ref()
                                            .snark_proof_with_public_inputs
                                            .clone();
                                        return Some(snark_task);
                                    }
                                }
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
