
use std::time::{SystemTime, UNIX_EPOCH};  
use std::time::Duration;  
use std::sync::Mutex;  
use once_cell::sync::OnceCell; 
  
fn get_current_timestamp() -> Result<u64, std::io::Error> {  
    let now = SystemTime::now();  
    let duration_since_epoch = now.duration_since(UNIX_EPOCH).unwrap();  
    let timestamp = duration_since_epoch.as_secs();  
    Ok(timestamp)
}  

pub static ACTIVE:u32 = 0;
pub static BUSY:u32 = 1;
pub static UNKNOW:u32 = 2;
pub static ACTIVE_TIMEOUT: u64 = 10;

#[derive(Debug, Clone)]
pub struct ProverNode {
    pub addr: String,
    pub state: u32,
    pub last_updated: u64,
}

impl ProverNode {
    pub fn new(addr: &String) -> Self {
        let prover_node = ProverNode {
            addr: addr.to_string(),
            state: UNKNOW,
            last_updated: 0,
        };
        prover_node
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
        return self.prover_nodes.clone();
    } 

    pub fn add_snark_node(&mut self, node: ProverNode) {  
        self.snark_nodes.push(node);
    }  
  
    pub fn get_snark_nodes(&self) -> Vec<ProverNode> {  
        return self.snark_nodes.clone();
    } 
}