use common::file;
use common::tls::Config as TlsConfig;
use prover_service::prover_service_client::ProverServiceClient;
use prover_service::AggregateAllRequest;
use prover_service::AggregateRequest;
use prover_service::FinalProofRequest;
use prover_service::GetTaskResultRequest;
use prover_service::ProveRequest;
use prover_service::SplitElfRequest;

use stage::tasks::{
    AggAllTask, AggTask, FinalTask, ProveTask, SplitTask, TASK_STATE_FAILED, TASK_STATE_PROCESSING,
    TASK_STATE_SUCCESS, TASK_STATE_UNPROCESSED, TASK_TIMEOUT,
};
use tonic::Request;

use self::prover_service::ResultCode;
use crate::prover_client::prover_service::AggregateInput;
use crate::prover_node::ProverNode;
use prover_service::GetTaskResultResponse;
use std::time::Duration;
use tonic::transport::Channel;

pub mod prover_service {
    tonic::include_proto!("prover.v1");
}

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

pub async fn get_snark_client(
    tls_config: Option<TlsConfig>,
) -> Option<(String, ProverServiceClient<Channel>)> {
    let nodes: Vec<ProverNode> = get_snark_nodes();
    for mut node in nodes {
        let client = node.is_active(tls_config.clone()).await;
        if let Some(client) = client {
            return Some((node.addr.clone(), client));
        }
    }
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
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
    let client = get_idle_client(tls_config).await;
    if let Some((addrs, mut client)) = client {
        let request = SplitElfRequest {
            chain_id: 0,
            timestamp: 0,
            proof_id: split_task.proof_id.clone(),
            computed_request_id: split_task.task_id.clone(),
            base_dir: split_task.base_dir.clone(),
            elf_path: split_task.elf_path.clone(),
            seg_path: split_task.seg_path.clone(),
            args: split_task.args.clone(),
            block_no: split_task.block_no,
            seg_size: split_task.seg_size,
        };
        log::info!("split request {:#?}", request);
        let mut grpc_request = Request::new(request);
        grpc_request.set_timeout(Duration::from_secs(TASK_TIMEOUT));
        let response = client.split_elf(grpc_request).await;
        if let Ok(response) = response {
            if let Some(response_result) = response.get_ref().result.as_ref() {
                log::info!("split response {:#?}", response);
                split_task.state = result_code_to_state(response_result.code);
                split_task.node_info = addrs;
                log::info!(
                    "split result proof_id:{:?} computed_request_id:{:?} code:{:?} message:{:?}",
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
        grpc_request.set_timeout(Duration::from_secs(TASK_TIMEOUT));
        let response = client.prove(grpc_request).await;
        if let Ok(response) = response {
            if let Some(response_result) = response.get_ref().result.as_ref() {
                log::info!("prove response {:#?}", response);
                prove_task.state = result_code_to_state(response_result.code);
                prove_task.node_info = addrs;
                log::info!(
                    "prove result proof_id:{:?} computed_request_id:{:?} code:{:?} message:{:?}",
                    response.get_ref().proof_id,
                    response.get_ref().computed_request_id,
                    response_result.code,
                    response_result.message,
                );
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
            chain_id: 0,
            timestamp: 0,
            proof_id: agg_task.proof_id.clone(),
            computed_request_id: agg_task.task_id.clone(),
            base_dir: agg_task.base_dir.clone(),
            seg_path: "".to_string(),
            block_no: agg_task.block_no,
            seg_size: agg_task.seg_size,
            input1: Some(AggregateInput {
                proof_path: agg_task.input1.proof_path.clone(),
                pub_value_path: agg_task.input1.pub_value_path.clone(),
                is_agg: agg_task.input1.is_agg,
            }),
            input2: Some(AggregateInput {
                proof_path: agg_task.input2.proof_path.clone(),
                pub_value_path: agg_task.input2.pub_value_path.clone(),
                is_agg: agg_task.input2.is_agg,
            }),
            agg_proof_path: agg_task.output_proof_path.clone(),
            agg_pub_value_path: agg_task.output_pub_value_path.clone(),
            output_dir: agg_task.output_dir.clone(),
            is_final: agg_task.is_final,
        };
        log::info!("aggregate request {:#?}", request);
        let mut grpc_request = Request::new(request);
        grpc_request.set_timeout(Duration::from_secs(TASK_TIMEOUT));
        let response = client.aggregate(grpc_request).await;
        if let Ok(response) = response {
            if let Some(response_result) = response.get_ref().result.as_ref() {
                log::info!("aggregate response {:#?}", response);
                agg_task.state = result_code_to_state(response_result.code);
                agg_task.node_info = addrs;
                log::info!(
                    "aggregate result proof_id:{:?} computed_request_id:{:?} code:{:?} message:{:?}",
                    response.get_ref().proof_id,
                    response.get_ref().computed_request_id,
                    response_result.code,
                    response_result.message,
                );
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
        log::info!("aggregate_all request {:#?}", request);
        let mut grpc_request = Request::new(request);
        grpc_request.set_timeout(Duration::from_secs(TASK_TIMEOUT));
        let response = client.aggregate_all(grpc_request).await;
        if let Ok(response) = response {
            if let Some(response_result) = response.get_ref().result.as_ref() {
                log::info!("aggregate_all response {:#?}", response);
                agg_all_task.state = result_code_to_state(response_result.code);
                agg_all_task.node_info = addrs;
                log::info!(
                    "aggregate_all result proof_id:{:?} computed_request_id:{:?} code:{:?} message:{:?}",
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

pub async fn final_proof(
    mut final_task: FinalTask,
    _tls_config: Option<TlsConfig>,
) -> Option<FinalTask> {
    let client = get_snark_client(None).await;
    if let Some((addrs, mut client)) = client {
        let (
            common_circuit_data_file,
            verifier_only_circuit_data_file,
            proof_with_public_inputs_file,
        ) = if final_task.input_dir.ends_with('/') {
            (
                format!("{}common_circuit_data.json", final_task.input_dir),
                format!("{}verifier_only_circuit_data.json", final_task.input_dir),
                format!("{}proof_with_public_inputs.json", final_task.input_dir),
            )
        } else {
            (
                format!("{}/common_circuit_data.json", final_task.input_dir),
                format!("{}/verifier_only_circuit_data.json", final_task.input_dir),
                format!("{}/proof_with_public_inputs.json", final_task.input_dir),
            )
        };
        let common_circuit_data = file::new(&common_circuit_data_file).read().unwrap();
        let verifier_only_circuit_data =
            file::new(&verifier_only_circuit_data_file).read().unwrap();
        let proof_with_public_inputs = file::new(&proof_with_public_inputs_file).read().unwrap();
        let request = FinalProofRequest {
            chain_id: 0,
            timestamp: 0,
            proof_id: final_task.proof_id.clone(),
            computed_request_id: final_task.task_id.clone(),
            common_circuit_data,
            proof_with_public_inputs,
            verifier_only_circuit_data,
        };
        log::info!(
            "final_proof request proof_id:{} computed_request_id:{} ",
            request.proof_id,
            request.computed_request_id,
        );
        let mut grpc_request = Request::new(request);
        grpc_request.set_timeout(Duration::from_secs(TASK_TIMEOUT));
        let response = client.final_proof(grpc_request).await;
        if let Ok(response) = response {
            if let Some(response_result) = response.get_ref().result.as_ref() {
                log::info!("final_proof response {:#?}", response);
                if ResultCode::from_i32(response_result.code) == Some(ResultCode::Ok) {
                    let mut loop_count = 0;
                    loop {
                        let task_result =
                            get_task_result(&mut client, &final_task.proof_id, &final_task.task_id)
                                .await;
                        if let Some(task_result) = task_result {
                            if let Some(result) = task_result.result {
                                if let Some(code) = ResultCode::from_i32(result.code) {
                                    if code == ResultCode::Ok {
                                        log::info!("final_proof result {:#?}", result);
                                        let _ = file::new(&final_task.output_path)
                                            .write(result.message.as_bytes())
                                            .unwrap();
                                        final_task.state = TASK_STATE_SUCCESS;
                                        final_task.node_info = addrs;
                                        return Some(final_task);
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
        final_task.state = TASK_STATE_FAILED;
    } else {
        final_task.state = TASK_STATE_UNPROCESSED;
    }
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    Some(final_task)
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
