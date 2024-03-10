use prover_service::prover_service::prover_service_server::ProverServiceServer;
use tonic::transport::Server;
mod prover_service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:5555".parse()?;
    let mut prover = prover_service::ProverServiceSVC::default();
    Server::builder()
        .add_service(ProverServiceServer::new(prover))
        .serve(addr)
        .await?;
    Ok(())
}
