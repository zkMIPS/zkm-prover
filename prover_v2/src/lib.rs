use std::time::Duration;
use zkm2_core_executor::ZKMContextBuilder;
use zkm2_core_machine::io::ZKMStdin;
use zkm2_prover::components::{DefaultProverComponents, ZKMProverComponents};
use zkm2_prover::{ZKMProver, ZKMProvingKey};
use zkm2_sdk::Prover;
use zkm2_stark::{ZKMCoreOpts, ZKMProverOpts};

pub mod agg_prover;
pub mod executor;
pub mod root_prover;
pub mod snark_prover;
pub mod contexts;

pub mod utils;
pub mod pipeline;

pub struct NetworkProve<'a, C: ZKMProverComponents = DefaultProverComponents> {
    // pub struct NetworkProve<'a> {
    pub prover: ZKMProver<C>,
    pub context_builder: ZKMContextBuilder<'a>,
    // pub pk: &'a ZKMProvingKey,
    pub stdin: ZKMStdin,
    pub opts: ZKMProverOpts,
    pub timeout: Option<Duration>,
}

impl<'a> NetworkProve<'a> {
    pub fn new() -> Self {
        Self {
            prover: ZKMProver::new(),
            context_builder: Default::default(),
            stdin: ZKMStdin::new(),
            opts: Default::default(),
            timeout: None,
        }
    }
}
