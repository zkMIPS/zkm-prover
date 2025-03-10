use crate::contexts::{AggAllContext, AggContext, ProveContext, SnarkContext};
use crate::provers::{AggAllProver, AggProver, Prover, RootProver, SnarkProver};

use crate::executor::{Executor, SplitContext};
use std::sync::Mutex;

#[derive(Default)]
pub struct Pipeline {
    mutex: Mutex<usize>,
    executor: Executor,
    root_prover: RootProver,
    agg_prover: AggProver,
    agg_all_prover: AggAllProver,
    snark_prover: SnarkProver,
}

impl Pipeline {
    pub fn new(base_dir: &str, proving_key_path: &str) -> Self {
        Pipeline {
            mutex: Mutex::new(0),
            executor: Executor::default(),
            root_prover: RootProver::default(),
            agg_prover: AggProver::default(),
            agg_all_prover: AggAllProver::default(),
            snark_prover: SnarkProver::new(
                proving_key_path.to_string(),
                format!("{}/input", base_dir),
                format!("{}/output", base_dir),
            ),
        }
    }

    pub fn split(&self, split_context: &mut SplitContext) -> Result<u64, String> {
        self.executor.split(split_context)
    }

    pub fn prove_root(
        &self,
        prove_context: &mut ProveContext,
    ) -> std::result::Result<bool, String> {
        let result = self.mutex.try_lock();
        match result {
            Ok(_) => match self.root_prover.prove(prove_context) {
                Ok(()) => Ok(true),
                Err(e) => {
                    log::error!("prove_root error {:#?}", e);
                    Err(e.to_string())
                }
            },
            Err(_e) => {
                log::error!("prove_root busy");
                Ok(false)
            }
        }
    }

    pub fn prove_aggregate(
        &mut self,
        agg_context: &mut AggContext,
    ) -> std::result::Result<bool, String> {
        let result = self.mutex.try_lock();
        match result {
            Ok(_) => match self.agg_prover.prove(agg_context) {
                Ok(()) => Ok(true),
                Err(e) => {
                    log::error!("prove_aggregate error {:#?}", e);
                    Err(e.to_string())
                }
            },
            Err(_) => {
                log::error!("prove_aggregate busy");
                Ok(false)
            }
        }
    }

    pub fn prove_snark(
        &mut self,
        snark_context: &mut SnarkContext,
    ) -> std::result::Result<bool, String> {
        let result = self.mutex.try_lock();
        match result {
            Ok(_) => match self.snark_prover.prove(snark_context) {
                Ok(()) => Ok(true),
                Err(e) => {
                    log::error!("prove_snark error {:#?}", e);
                    Err(e.to_string())
                }
            },
            Err(_) => {
                log::error!("prove_snark busy");
                Ok(false)
            }
        }
    }

    pub fn prove_aggregate_all(
        &mut self,
        final_context: &mut AggAllContext,
    ) -> std::result::Result<bool, String> {
        let result = self.mutex.try_lock();
        match result {
            Ok(_) => match self.agg_all_prover.prove(final_context) {
                Ok(()) => Ok(true),
                Err(e) => {
                    log::error!("prove_aggregate_all error {:#?}", e);
                    Err(e.to_string())
                }
            },
            Err(_) => {
                log::error!("prove_aggregate_all busy");
                Ok(false)
            }
        }
    }

    /// Return zkm-prover status
    pub fn get_status(&self) -> bool {
        let result = self.mutex.try_lock();
        result.is_ok()
    }
}
