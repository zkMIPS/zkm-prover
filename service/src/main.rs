use clap::{Command, Arg, Parser};
use prover_node::ProverNode;  
use std::io;

use prover_service::prover_service::prover_service_server::ProverServiceServer;
use stage_service::stage_service::stage_service_server::StageServiceServer;
use tonic::transport::Server;

mod prover_service;
mod stage_service;
mod prover_client;
mod prover_node;
mod config;


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = String::from("./config.toml"))]
    config: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let conf_path = std::path::Path::new(&args.config);
    let runtime_config = config::RuntimeConfig::from_toml(conf_path).expect("Config is missing");
    let addr = runtime_config.addr.as_str().parse()?;
    let nodes_lock = crate::prover_node::instance();
    {
        let mut nodes_data = nodes_lock.lock().unwrap();
        for node in runtime_config.prover_addrs {
            nodes_data.add_node(ProverNode::new(&node));
        }
    }
    let mut prover = prover_service::ProverServiceSVC::default();
    let mut stage = stage_service::StageServiceSVC::default();
    Server::builder()
        .add_service(ProverServiceServer::new(prover))
        .add_service(StageServiceServer::new(stage))
        .serve(addr)
        .await?;
    Ok(())
}
