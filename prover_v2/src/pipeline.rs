use std::sync::Mutex;

use crate::agg_prover::AggProver;
use crate::contexts::{AggContext, ProveContext, SnarkContext, SplitContext};
use crate::executor::Executor;
use crate::root_prover::RootProver;
use crate::snark_prover::SnarkProver;

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
        self.executor
            .split(split_context)
            .map(|n| (true, n))
            .map_err(|e| e.to_string())
    }

    pub fn prove_root(&self, prove_context: &ProveContext) -> Result<(bool, Vec<u8>), String> {
        match self.mutex.try_lock() {
            Ok(_) => self
                .root_prover
                .prove(prove_context)
                .map(|receipt_output| (true, receipt_output))
                .map_err(|e| {
                    tracing::error!("prove_root error {:#?}", e);
                    e.to_string()
                }),
            Err(e) => {
                tracing::error!("prove_root busy: {:?}", e);
                Ok((false, vec![]))
            }
        }
    }

    pub fn prove_aggregate(&self, agg_context: &AggContext) -> Result<(bool, Vec<u8>), String> {
        match self.mutex.try_lock() {
            Ok(_) => self
                .agg_prover
                .prove(agg_context)
                .map(|agg_receipt_output| (true, agg_receipt_output))
                .map_err(|e| {
                    tracing::error!("prove_aggregate error {:#?}", e);
                    e.to_string()
                }),
            Err(e) => {
                tracing::error!("prove_aggregate busy: {:?}", e);
                Ok((false, vec![]))
            }
        }
    }

    pub fn prove_snark(&self, snark_context: &SnarkContext) -> Result<(bool, Vec<u8>), String> {
        match self.mutex.try_lock() {
            Ok(_) => self.snark_prover.prove(snark_context).map_err(|e| {
                tracing::error!("prove_snark error {:#?}", e);
                e.to_string()
            }),
            Err(e) => {
                tracing::error!("prove_snark busy: {:?}", e);
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
