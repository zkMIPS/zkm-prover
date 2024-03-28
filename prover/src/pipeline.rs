use crate::contexts::{AggAllContext, AggContext, ProveContext};
use crate::provers::{AggAllProver, AggProver, Prover, RootProver};

// use anyhow::{anyhow, bail, Result};
// use std::path::Path;
use std::sync::Mutex;

#[derive(Debug, Default)]
pub struct Pipeline {
    _mutex: Mutex<usize>,
}

static PIPELINE_MUTEX: Mutex<usize> = Mutex::new(0);

impl Pipeline {
    pub fn new() -> Self {
        Pipeline {
            _mutex: Mutex::new(0),
        }
    }

    pub fn prove_root(
        &mut self,
        prove_context: &ProveContext,
    ) -> std::result::Result<bool, String> {
        let result = PIPELINE_MUTEX.try_lock();
        match result {
            Ok(_) => match RootProver::new().prove(prove_context) {
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
        agg_context: &AggContext,
    ) -> std::result::Result<bool, String> {
        let result = PIPELINE_MUTEX.try_lock();
        match result {
            Ok(_) => match AggProver::new().prove(agg_context) {
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

    pub fn prove_aggregate_all(
        &mut self,
        final_context: &AggAllContext,
    ) -> std::result::Result<bool, String> {
        let result = PIPELINE_MUTEX.try_lock();
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

    /// Return prover status
    pub fn get_status(&mut self) -> bool {
        let result = PIPELINE_MUTEX.try_lock();
        result.is_ok()
    }
}
