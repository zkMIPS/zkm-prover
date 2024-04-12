use once_cell::sync::OnceCell;
use std::sync::{Arc, Mutex};

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

    pub fn set_client(&mut self, client: Option<tonic::transport::channel::Channel>) {
        *self.client.lock().unwrap() = client;
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
