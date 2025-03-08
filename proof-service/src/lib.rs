mod config;
mod database;
mod executor;
mod metrics;
mod prover_client;
mod prover_node;
mod prover_service;
mod stage;

pub mod proto;

use common::tls::Config as TlsConfig;
