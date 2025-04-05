use std::time::Duration;
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

pub struct NetworkProve<'a, C: ZKMProverComponents = DefaultProverComponents> {
    pub prover: ZKMProver<C>,
    pub context_builder: ZKMContextBuilder<'a>,
    pub stdin: ZKMStdin,
    pub opts: ZKMProverOpts,
    pub timeout: Option<Duration>,
}

// TODO: create from config file
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

    pub fn new_with_segment_size(segment_size: u32) -> Self {
        let mut opts = ZKMProverOpts::default();
        if segment_size > 0 {
            opts.core_opts.shard_size = segment_size as usize;
        }

        Self {
            prover: ZKMProver::new(),
            context_builder: Default::default(),
            stdin: ZKMStdin::new(),
            opts,
            timeout: None,
        }
    }
}
