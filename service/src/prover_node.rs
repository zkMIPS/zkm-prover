use crate::prover_client::prover_service::get_status_response;
use crate::prover_client::prover_service::prover_service_client::ProverServiceClient;
use crate::prover_client::prover_service::GetStatusRequest;
use common::tls::Config as TlsConfig;
use once_cell::sync::OnceCell;
use stage::tasks::TASK_TIMEOUT;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tonic::transport::Channel;
use tonic::transport::ClientTlsConfig;
use tonic::transport::Uri;
use tonic::Request;

#[derive(Debug, Clone)]
pub struct ProverNode {
    pub addr: String,
    pub client: Arc<Mutex<Option<tonic::transport::channel::Channel>>>,
}

impl ProverNode {
    pub fn new(addr: &String) -> Self {
        ProverNode {
            addr: addr.to_string(),
            client: Arc::new(Mutex::new(None)),
        }
    }

    pub fn get_client(&self) -> Option<tonic::transport::channel::Channel> {
        self.client.lock().unwrap().clone()
    }

    fn set_client(&mut self, client: Option<tonic::transport::channel::Channel>) {
        *self.client.lock().unwrap() = client;
    }

    pub async fn is_active(
        &mut self,
        tls_config: Option<TlsConfig>,
    ) -> Option<ProverServiceClient<Channel>> {
        let mut client = self.get_client();
        match client {
            Some(_) => {}
            None => {
                let uri = format!("grpc://{}", self.addr).parse::<Uri>().unwrap();
                let mut endpoint = tonic::transport::Channel::builder(uri)
                    .connect_timeout(Duration::from_secs(5))
                    .timeout(Duration::from_secs(TASK_TIMEOUT))
                    .concurrency_limit(256);
                if let Some(config) = tls_config {
                    let mut tls_config = ClientTlsConfig::new();
                    if let Some(ca_cert) = config.ca_cert {
                        tls_config = tls_config.ca_certificate(ca_cert);
                    }
                    if let Some(identity) = config.identity {
                        tls_config = tls_config.identity(identity);
                    }
                    endpoint = endpoint.tls_config(tls_config).unwrap();
                }
                let client_init = endpoint.connect().await;
                if let Ok(client_init) = client_init {
                    self.set_client(Some(client_init.clone()));
                    client = Some(client_init.clone());
                }
            }
        }

        if let Some(client) = client {
            let mut client = ProverServiceClient::<Channel>::new(client);
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
            } else {
                self.set_client(None);
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct ProverNodes {
    pub prover_nodes: Vec<ProverNode>,
    pub snark_nodes: Vec<ProverNode>,
}

static INSTANCE: OnceCell<Mutex<ProverNodes>> = OnceCell::new();

pub fn instance() -> &'static Mutex<ProverNodes> {
    INSTANCE.get_or_init(|| Mutex::new(ProverNodes::new()))
}

impl ProverNodes {
    fn new() -> Self {
        ProverNodes {
            prover_nodes: Vec::new(),
            snark_nodes: Vec::new(),
        }
    }
    pub fn add_node(&mut self, node: ProverNode) {
        self.prover_nodes.push(node);
    }

    pub fn get_nodes(&self) -> Vec<ProverNode> {
        self.prover_nodes.clone()
    }

    pub fn add_snark_node(&mut self, node: ProverNode) {
        self.snark_nodes.push(node);
    }

    pub fn get_snark_nodes(&self) -> Vec<ProverNode> {
        self.snark_nodes.clone()
    }
}
