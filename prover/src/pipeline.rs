use crate::contexts::{AggContext, AggAllContext, ProveContext, SplitContext};
use crate::provers::{Executor, RootProver, AggProver, AggAllProver, Prover};

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

    pub fn split(&mut self, split_context: &SplitContext) -> bool {
        let result = PIPELINE_MUTEX.try_lock();
        match result {
            Ok(_) => {
                match Executor::new().split(split_context) {
                    Ok(()) => {
                        true
                    }
                    Err(e) => {
                        log::error!("split error {:#?}", e);
                        false
                    }   
                }
            }
            Err(_) => {
                log::error!("split_prove busy");
                false
            }
        }
    }

    pub fn prove_root(&mut self, prove_context: &ProveContext) -> bool {
        let result = PIPELINE_MUTEX.try_lock();
        match result {
            Ok(_) => {
                match RootProver::new().prove(prove_context) {
                    Ok(()) => {
                        true
                    }
                    Err(e) => {
                        log::error!("prove_root error {:#?}", e);
                        false
                    }   
                }
            }
            Err(_e) => {
                log::error!("prove_root busy");
                false
            }
        }
    }

    pub fn prove_aggregate(&mut self, agg_context: &AggContext) -> bool {
        let result = PIPELINE_MUTEX.try_lock();
        match result {
            Ok(_) => {
                match AggProver::new().prove(agg_context) {
                    Ok(()) => {
                        true
                    }
                    Err(e) => {
                        log::error!("prove_aggregate error {:#?}", e);
                        false
                    }   
                }
            }
            Err(_) => {
                log::error!("prove_aggregate busy");
                false
            }
        }
    }

    pub fn prove_aggregate_all(&mut self, final_context: &AggAllContext) -> bool {
        let result = PIPELINE_MUTEX.try_lock();
        match result {
            Ok(_) => {
                match AggAllProver::new().prove(final_context) {
                    Ok(()) => {
                        true
                    }
                    Err(e) => {
                        log::error!("prove_aggregate_all error {:#?}", e);
                        false
                    }   
                }
            }
            Err(_) => {
                log::error!("prove_aggregate_all busy");
                false
            }
        }
    }

    /// Return prover status
    pub fn get_status(&mut self) -> bool {
        let result = PIPELINE_MUTEX.try_lock();
        match result {
            Ok(_) => {
                true
            }
            Err(_) => {
                false
            }
        }
    }
}