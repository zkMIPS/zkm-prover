use clap::Parser;
use prover_node::ProverNode;

use common::tls::Config as TlsConfig;
use prover_service::prover_service::prover_service_server::ProverServiceServer;
use stage_service::stage_service::stage_service_server::StageServiceServer;
use tonic::transport::Server;
use tonic::transport::ServerTlsConfig;

mod config;
mod prover_client;
mod prover_node;
mod prover_service;
mod stage_service;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'c', long = "config", default_value_t = String::from("./config/config.toml"))]
    config: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::try_init().unwrap_or_default();
    let args = Args::parse();
    let conf_path = std::path::Path::new(&args.config);
    let runtime_config = config::RuntimeConfig::from_toml(conf_path).expect("Config is missing");
    let addr = runtime_config.addr.as_str().parse()?;
    let nodes_lock = crate::prover_node::instance();
    {
        let mut nodes_data = nodes_lock.lock().unwrap();
        for node in &runtime_config.prover_addrs {
            nodes_data.add_node(ProverNode::new(node));
        }
        for node in &runtime_config.snark_addrs {
            nodes_data.add_snark_node(ProverNode::new(node));
        }
    }
    let prover = prover_service::ProverServiceSVC::default();
    let stage = stage_service::StageServiceSVC::new(runtime_config.clone()).await?;
    if runtime_config.ca_cert_path.is_some() {
        let tls_config = TlsConfig::new(
            runtime_config.ca_cert_path.unwrap(),
            runtime_config.cert_path.unwrap(),
            runtime_config.key_path.unwrap(),
        )
        .await?;
        Server::builder()
            .tls_config(
                ServerTlsConfig::new()
                    .identity(tls_config.identity)
                    .client_ca_root(tls_config.ca_cert),
            )?
            .add_service(ProverServiceServer::new(prover))
            .add_service(StageServiceServer::new(stage))
            .serve(addr)
            .await?;
    } else {
        Server::builder()
            .add_service(ProverServiceServer::new(prover))
            .add_service(StageServiceServer::new(stage))
            .serve(addr)
            .await?;
    }

    Ok(())
}
