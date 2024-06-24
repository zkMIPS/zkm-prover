use common::file;
use log::error;
use once_cell::sync::OnceCell;
use serde_derive::Deserialize;
use std::sync::Mutex;

static INSTANCE: OnceCell<Mutex<RuntimeConfig>> = OnceCell::new();

pub fn instance() -> &'static Mutex<RuntimeConfig> {
    INSTANCE.get_or_init(|| Mutex::new(RuntimeConfig::new()))
}

#[derive(Debug, Deserialize, Clone)]
pub struct RuntimeConfig {
    pub addr: String,
    pub metrics_addr: String,
    pub database_url: String,
    pub prover_addrs: Vec<String>,
    pub snark_addrs: Vec<String>,
    pub base_dir: String,
    pub fileserver_url: Option<String>,
    pub ca_cert_path: Option<String>,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
}

impl RuntimeConfig {
    pub fn new() -> Self {
        RuntimeConfig {
            addr: "0.0.0.0:50000".to_string(),
            metrics_addr: "0.0.0.0:50010".to_string(),
            database_url: "mysql://user:password@localhost:3306/dbname".to_string(),
            prover_addrs: ["0.0.0.0:50000".to_string()].to_vec(),
            snark_addrs: ["0.0.0.0:50000".to_string()].to_vec(),
            base_dir: "/tmp".to_string(),
            fileserver_url: None,
            ca_cert_path: None,
            cert_path: None,
            key_path: None,
        }
    }

    pub fn from_toml(path: &str) -> Option<Self> {
        let contents = match file::new(path).read_to_string() {
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
        // both of ca_cert_path, cert_path, key_path should be some or none
        // if (config.ca_cert_path.is_some()
        //     || config.cert_path.is_some()
        //     || config.key_path.is_some())
        //     && (config.ca_cert_path.is_none()
        //         || config.cert_path.is_none()
        //         || config.key_path.is_none())
        // {
        //     error!("both of ca_cert_path, cert_path, key_path should be some or none");
        //     return None;
        // }
        instance().lock().unwrap().addr.clone_from(&config.addr);
        instance()
            .lock()
            .unwrap()
            .metrics_addr
            .clone_from(&config.metrics_addr);
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
            .fileserver_url
            .clone_from(&config.fileserver_url);
        instance()
            .lock()
            .unwrap()
            .snark_addrs
            .clone_from(&config.snark_addrs);
        instance()
            .lock()
            .unwrap()
            .ca_cert_path
            .clone_from(&config.ca_cert_path);
        instance()
            .lock()
            .unwrap()
            .cert_path
            .clone_from(&config.cert_path);
        instance()
            .lock()
            .unwrap()
            .key_path
            .clone_from(&config.key_path);
        Some(config)
    }
}
