use prover_service::prover_service::prover_service_server::ProverServiceServer;
use stage_service::stage_service::stage_service_server::StageServiceServer;
use tonic::transport::Server;
mod prover_service;
mod stage_service;
mod prover_client;
mod prover_node;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:5555".parse()?;
    let mut prover = prover_service::ProverServiceSVC::default();
    let mut stage = stage_service::StageServiceSVC::default();
    Server::builder()
        .add_service(ProverServiceServer::new(prover))
        .add_service(StageServiceServer::new(stage))
        .serve(addr)
        .await?;
    Ok(())
}
