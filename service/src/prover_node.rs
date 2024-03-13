
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
    pub ip: String,
    pub port: u16,
    pub state: u32,
    pub last_updated: u64,
    // client
}

impl ProverNode {
    pub fn new(ip: &String, port: u16) -> Self {
        let prover_node = ProverNode {
            ip: ip.to_string(),
            port,
            state: UNKNOW,
            last_updated: 0,
        };
        // client init
        prover_node
    }

    pub fn is_active(&mut self) -> bool {
        if self.state != ACTIVE {
            return false
        }
        let now = get_current_timestamp().unwrap();
        if self.last_updated + ACTIVE_TIMEOUT < now {
            return false
        }
        true
    }

    pub fn update_state(&mut self, state: u32) {
        self.state = state;
        self.last_updated = get_current_timestamp().unwrap();
    }
}

#[derive(Debug)]
pub struct ProverNodes {
    pub prover_nodes: Vec<ProverNode>,
}

static INSTANCE: OnceCell<Mutex<ProverNodes>> = OnceCell::new();
  
pub fn instance() -> &'static Mutex<ProverNodes> {  
    INSTANCE.get_or_init(|| Mutex::new(ProverNodes::new()))
} 

impl ProverNodes {
    fn new() -> Self {
        ProverNodes {
            prover_nodes: Vec::new(),
        }
    }
    pub fn set_nodes(&mut self, nodes: Vec<ProverNode>) {  
        self.prover_nodes = nodes;
    }  
  
    pub fn get_nodes(&self) -> Vec<ProverNode> {  
        return self.prover_nodes.clone();
    } 
}