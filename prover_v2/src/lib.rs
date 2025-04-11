use once_cell::sync::OnceCell;
use std::sync::Mutex;
use std::time::Duration;
use zkm_core_executor::ZKMContextBuilder;
use zkm_core_machine::io::ZKMStdin;
use zkm_prover::{OuterSC, ZKMProver};
use zkm_stark::{StarkProvingKey, StarkVerifyingKey, ZKMProverOpts};

pub mod agg_prover;
pub mod contexts;
pub mod executor;
pub mod root_prover;
pub mod snark_prover;

pub mod pipeline;

#[derive(Default)]
pub struct NetworkProve<'a> {
    pub context_builder: ZKMContextBuilder<'a>,
    pub stdin: ZKMStdin,
    pub opts: ZKMProverOpts,
    pub timeout: Option<Duration>,
}

static GLOBAL_PROVER: OnceCell<Mutex<ZKMProver>> = OnceCell::new();
fn prover_instance() -> &'static Mutex<ZKMProver> {
    GLOBAL_PROVER.get_or_init(|| Mutex::new(ZKMProver::new()))
}

pub fn get_prover() -> impl std::ops::DerefMut<Target = ZKMProver> {
    prover_instance()
        .lock()
        .expect("GLOBAL_PROVER lock poisoned")
}

static WRAP_KEYS: OnceCell<(StarkProvingKey<OuterSC>, StarkVerifyingKey<OuterSC>)> =
    OnceCell::new();
