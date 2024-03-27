use log::error;
use once_cell::sync::OnceCell;
use serde_derive::Deserialize;
use std::fs;
use std::path::Path;
use std::sync::Mutex;

static INSTANCE: OnceCell<Mutex<RuntimeConfig>> = OnceCell::new();

pub fn instance() -> &'static Mutex<RuntimeConfig> {
    INSTANCE.get_or_init(|| Mutex::new(RuntimeConfig::new()))
}

#[derive(Debug, Deserialize)]
pub struct RuntimeConfig {
    pub addr: String,
    pub prover_addrs: Vec<String>,
    pub snark_addrs: Vec<String>,
    pub base_dir: String,
}

impl RuntimeConfig {
    pub fn new() -> Self {
        RuntimeConfig {
            addr: "0.0.0.0:50000".to_string(),
            prover_addrs: ["0.0.0.0:50000".to_string()].to_vec(),
            snark_addrs: ["0.0.0.0:50000".to_string()].to_vec(),
            base_dir: "/tmp".to_string(),
        }
    }

    pub fn from_toml<T: AsRef<Path>>(path: T) -> Option<Self> {
        let contents = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                error!(
                    "Something went wrong reading the runtime config file, {:?}",
                    e
                );
                return None;
            }
        };
        let config: RuntimeConfig = match toml::from_str(&contents) {
            Ok(c) => c,
            Err(e) => {
                error!(
                    "Something went wrong reading the runtime config file, {:?}",
                    e
                );
                return None;
            }
        };
        instance().lock().unwrap().addr.clone_from(&config.addr);
        instance()
            .lock()
            .unwrap()
            .prover_addrs
            .clone_from(&config.prover_addrs);
        instance()
            .lock()
            .unwrap()
            .base_dir
            .clone_from(&config.base_dir);
        instance()
            .lock()
            .unwrap()
            .snark_addrs
            .clone_from(&config.snark_addrs);
        Some(config)
    }
}
