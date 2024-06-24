use clap::Parser;
use prover_node::ProverNode;

use common::tls::Config as TlsConfig;
use prover_service::prover_service::prover_service_server::ProverServiceServer;
use stage_service::stage_service::stage_service_server::StageServiceServer;
use tonic::transport::Server;
use tonic::transport::ServerTlsConfig;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response};
use prometheus::{Encoder, TextEncoder};

mod config;
mod database;
mod metrics;
mod prover_client;
mod prover_node;
mod prover_service;
mod stage_service;
mod stage_worker;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'c', long = "config", default_value_t = String::from("./config/config.toml"))]
    config: String,
    #[arg(short = 's', long = "stage", default_value_t = false)]
    stage: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::try_init().unwrap_or_default();
    let args = Args::parse();
    let runtime_config = config::RuntimeConfig::from_toml(&args.config).expect("Config is missing");
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
    let mut server = Server::builder();
    if runtime_config.key_path.is_some() {
        let tls_config = TlsConfig::new(
            runtime_config
                .ca_cert_path
                .clone()
                .unwrap_or("".to_string()),
            runtime_config.cert_path.clone().unwrap(),
            runtime_config.key_path.clone().unwrap(),
        )
        .await?;
        let mut server_tls_config = ServerTlsConfig::new();
        if let Some(ca_cert) = tls_config.ca_cert {
            server_tls_config = server_tls_config.client_ca_root(ca_cert);
        }
        if let Some(identity) = tls_config.identity {
            server_tls_config = server_tls_config.identity(identity);
        }
        server = server.tls_config(server_tls_config)?;
    }
    let grpc_server = if args.stage {
        let stage = stage_service::StageServiceSVC::new(runtime_config.clone()).await?;
        server
            .add_service(StageServiceServer::new(stage))
            .serve(addr)
    } else {
        let prover = prover_service::ProverServiceSVC::default();
        server
            .add_service(ProverServiceServer::new(prover))
            .serve(addr)
    };

    let metrics_addr = runtime_config.metrics_addr.as_str().parse()?;
    let make_svc = make_service_fn(move |_| {
        let registry = metrics::REGISTRY_INSTANCE.clone();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |_: Request<Body>| {
                let registry = registry.clone();
                async move {
                    let encoder = TextEncoder::new();
                    let metric_families = registry.gather();
                    let mut buffer = Vec::new();
                    encoder.encode(&metric_families, &mut buffer).unwrap();
                    Ok::<_, hyper::Error>(Response::new(Body::from(buffer)))
                }
            }))
        }
    });
    metrics::init_registry();
    let metrics_server = hyper::Server::bind(&metrics_addr).serve(make_svc);

    tokio::pin!(grpc_server);
    tokio::pin!(metrics_server);

    tokio::select! {
        res = grpc_server => res?,
        res = metrics_server => res?,
    }
    Ok(())
}
