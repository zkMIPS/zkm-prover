use std::borrow::BorrowMut;
use std::result;

use prover_service::prover_service_client::ProverServiceClient;
use prover_service::{Result};
use prover_service::{GetStatusRequest, GetStatusResponse};
use prover_service::{SplitElfRequest, SplitElfResponse};
use prover_service::{ProveRequest, ProveResponse};
use prover_service::{AggregateRequest, AggregateResponse};
use prover_service::{AggregateAllRequest, AggregateAllResponse};

use prover::pipeline::{self,Pipeline};

use tonic::{client, Request, Response, Status};
use stage::tasks::{SplitTask, ProveTask, AggTask, FinalTask};

use self::prover_service::ResultCode;
use tonic::transport::{Uri}; 
use tonic::transport::Channel;
use std::net::ToSocketAddrs;
use crate::prover_node::ProverNode;
use crate::prover_node::ProverNodes;

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
        let client = is_active(node.ip, node.port).await;
        if let Some(client) = client {
            return Some(client);
        }
    }
    return None;
}

pub async fn is_active(ip: String, port: u16) -> Option<ProverServiceClient<Channel>> {
    let uri = format!("grpc://{}:{}", ip, port).parse::<Uri>().unwrap();
    let endpoint = tonic::transport::Channel::builder(uri);
    let mut client = ProverServiceClient::connect(endpoint).await.unwrap();  
    let request = GetStatusRequest {};
    let response = client.get_status(Request::new(request)).await.unwrap();
    if response.get_ref().status == 0 {
        return Some(client);
    }  
    return None;
}

pub async fn split(split_task: SplitTask) -> Option<SplitTask> {
    let mut client = get_idle_client().await;  
    if let Some(mut client) = client {
        let request = SplitElfRequest::default();
        // TODO
        let response = client.split_elf(Request::new(request)).await.unwrap();
        if let Some(response_result) = response.get_ref().result.as_ref() {
            if ResultCode::from_i32(response_result.code) == Some(ResultCode::ResultOk) {
                // SUCCESS
                return Some(split_task);
            }
        }
    }
    Some(split_task)
}

pub async fn prove(prove_task: ProveTask) -> Option<ProveTask> {
    let mut client = get_idle_client().await;  
    if let Some(mut client) = client {
        let request = ProveRequest::default();
        // TODO
        let response = client.prove(Request::new(request)).await.unwrap();
        if let Some(response_result) = response.get_ref().result.as_ref() {
            if ResultCode::from_i32(response_result.code) == Some(ResultCode::ResultOk) {
                // SUCCESS
                return Some(prove_task);
            }
        }
    }
    Some(prove_task)
}

pub async fn aggregate(agg_task: AggTask) -> Option<AggTask> {
    let mut client = get_idle_client().await;  
    if let Some(mut client) = client {
        let request = AggregateRequest::default();
        // TODO
        let response = client.aggregate(Request::new(request)).await.unwrap();
        if let Some(response_result) = response.get_ref().result.as_ref() {
            if ResultCode::from_i32(response_result.code) == Some(ResultCode::ResultOk) {
                // SUCCESS
                return Some(agg_task);
            }
        }
    }
    Some(agg_task)
}

pub async fn aggregate_all(final_task: FinalTask) -> Option<FinalTask> {
    let mut client = get_idle_client().await;  
    if let Some(mut client) = client {
        let request = AggregateAllRequest::default();
        // TODO
        let response = client.aggregate_all(Request::new(request)).await.unwrap();
        if let Some(response_result) = response.get_ref().result.as_ref() {
            if ResultCode::from_i32(response_result.code) == Some(ResultCode::ResultOk) {
                // SUCCESS
                return Some(final_task);
            }
        }
    }
    Some(final_task)
}