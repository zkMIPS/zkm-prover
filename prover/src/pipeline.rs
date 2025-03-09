use crate::contexts::{AggAllContext, AggContext, ProveContext, SnarkContext};
use crate::provers::{AggAllProver, AggProver, Prover, RootProver, SnarkProver};

// use anyhow::{anyhow, bail, Result};
// use std::path::Path;
use std::sync::Mutex;
#[derive(Debug, Default)]
pub struct Pipeline {
    mutex: Mutex<usize>,
}

impl Pipeline {
    pub fn new() -> Self {
        Pipeline {
            mutex: Mutex::new(0),
        }
    }

    pub fn prove_root(
        &mut self,
        prove_context: &mut ProveContext,
    ) -> std::result::Result<bool, String> {
        let result = self.mutex.try_lock();
        match result {
            Ok(_) => match RootProver::default().prove(prove_context) {
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
            Ok(_) => match AggProver::default().prove(agg_context) {
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
            Ok(_) => match SnarkProver::default().prove(snark_context) {
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
            Ok(_) => match AggAllProver::new().prove(final_context) {
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
    pub fn get_status(&mut self) -> bool {
        let result = self.mutex.try_lock();
        result.is_ok()
    }
}
