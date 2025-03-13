use crate::contexts::{AggContext, ProveContext, SnarkContext};
use crate::provers::{AggProver, Prover, RootProver, SnarkProver};

use crate::executor::{Executor, SplitContext};
use std::sync::Mutex;

#[derive(Default)]
pub struct Pipeline {
    mutex: Mutex<usize>,
    executor: Executor,
    root_prover: RootProver,
    agg_prover: AggProver,
    snark_prover: SnarkProver,
}

impl Pipeline {
    pub fn new(base_dir: &str, keys_input_dir: &str) -> Self {
        Pipeline {
            mutex: Mutex::new(0),
            executor: Executor::default(),
            root_prover: RootProver::default(),
            agg_prover: AggProver::default(),
            snark_prover: SnarkProver::new(keys_input_dir, &format!("{}/output", base_dir)),
        }
    }

    pub fn split(&self, split_context: &SplitContext) -> Result<(bool, u64), String> {
        match self.executor.split(split_context) {
            Ok(n) => Ok((true, n)),
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn prove_root(
        &self,
        prove_context: &ProveContext,
    ) -> std::result::Result<(bool, Vec<u8>), String> {
        let result = self.mutex.try_lock();
        match result {
            Ok(_) => match self.root_prover.prove(prove_context) {
                Ok(receipt_output) => Ok(receipt_output),
                Err(e) => {
                    log::error!("prove_root error {:#?}", e);
                    Err(e.to_string())
                }
            },
            Err(e) => {
                log::error!("prove_root busy: {:?}", e);
                Ok((false, vec![]))
            }
        }
    }

    pub fn prove_aggregate(
        &mut self,
        agg_context: &AggContext,
    ) -> std::result::Result<(bool, Vec<u8>), String> {
        let result = self.mutex.try_lock();
        match result {
            Ok(_) => match self.agg_prover.prove(agg_context) {
                Ok(agg_receipt_output) => Ok(agg_receipt_output),
                Err(e) => {
                    log::error!("prove_aggregate error {:#?}", e);
                    Err(e.to_string())
                }
            },
            Err(e) => {
                log::error!("prove_aggregate busy: {:?}", e);
                Ok((false, vec![]))
            }
        }
    }

    pub fn prove_snark(
        &mut self,
        snark_context: &SnarkContext,
    ) -> std::result::Result<(bool, Vec<u8>), String> {
        let result = self.mutex.try_lock();
        match result {
            Ok(_) => match self.snark_prover.prove(snark_context) {
                Ok(output) => Ok(output),
                Err(e) => {
                    log::error!("prove_snark error {:#?}", e);
                    Err(e.to_string())
                }
            },
            Err(e) => {
                log::error!("prove_snark busy: {:?}", e);
                Ok((false, vec![]))
            }
        }
    }

    /// Return zkm-prover status
    pub fn get_status(&self) -> bool {
        let result = self.mutex.try_lock();
        result.is_ok()
    }
}
