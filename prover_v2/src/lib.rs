use std::sync::{Arc, Mutex};
use std::time::Duration;
use once_cell::sync::OnceCell;
use zkm2_core_executor::ZKMContextBuilder;
use zkm2_core_machine::io::ZKMStdin;
use zkm2_prover::components::{DefaultProverComponents, ZKMProverComponents};
use zkm2_prover::{ZKMProver, ZKMProvingKey};
use zkm2_sdk::Prover;
use zkm2_stark::{ZKMCoreOpts, ZKMProverOpts};

pub mod agg_prover;
pub mod contexts;
pub mod executor;
pub mod root_prover;
pub mod snark_prover;

pub mod pipeline;
pub mod utils;

pub struct NetworkProve<'a> {
    pub context_builder: ZKMContextBuilder<'a>,
    pub stdin: ZKMStdin,
    pub opts: ZKMProverOpts,
    pub timeout: Option<Duration>,
}

// TODO: create from config file
impl<'a> NetworkProve<'a> {
    pub fn new() -> Self {
        Self {
            context_builder: Default::default(),
            stdin: ZKMStdin::new(),
            opts: Default::default(),
            timeout: None,
        }
    }
}

static GLOBAL_PROVER: OnceCell<Mutex<ZKMProver>> = OnceCell::new();
fn prover_instance() -> &'static Mutex<ZKMProver> {
    GLOBAL_PROVER.get_or_init(|| Mutex::new(ZKMProver::new()))
}

pub fn get_prover() -> impl std::ops::DerefMut<Target=ZKMProver> {
    prover_instance().lock().expect("GLOBAL_PROVER lock poisoned")
}