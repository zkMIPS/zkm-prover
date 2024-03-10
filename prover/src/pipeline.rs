use crate::contexts::{agg_context, final_context, AggContext, FinalContext, ProveContext, SplitContext};
use crate::provers::{SplitProver, ProveProver, AggProver, FinalProver, Prover};

use anyhow::{anyhow, bail, Result};
use std::path::Path;
use std::sync::Mutex;

#[derive(Debug, Default)]
pub struct Pipeline {
    mutex: Mutex<usize>,
}

static PIPELINE_MUTEX: Mutex<usize> = Mutex::new(0);

impl Pipeline {
    pub fn new() -> Self {
        Pipeline {
            mutex: Mutex::new(0),
        }
    }

    pub fn split_prove(&mut self, split_context: &SplitContext) -> Result<String> {
        let result = PIPELINE_MUTEX.try_lock();
        match result {
            Ok(_) => {
                match SplitProver::new().prove(split_context) {
                    Ok(()) => {
                        Ok(String::from("SUCCESS"))
                    }
                    Err(_) => {
                        Ok(String::from("BUSY"))
                    }   
                }
            }
            Err(_) => {
                Ok(String::from("BUSY"))
            }
        }
    }

    pub fn root_prove(&mut self, prove_context: &ProveContext) -> Result<String> {
        let result = PIPELINE_MUTEX.try_lock();
        match result {
            Ok(_) => {
                match ProveProver::new().prove(prove_context) {
                    Ok(()) => {
                        Ok(String::from("SUCCESS"))
                    }
                    Err(_) => {
                        Ok(String::from("BUSY"))
                    }   
                }
            }
            Err(_) => {
                Ok(String::from("BUSY"))
            }
        }
    }

    pub fn aggregate_prove(&mut self, agg_context: &AggContext) -> Result<String> {
        let result = PIPELINE_MUTEX.try_lock();
        match result {
            Ok(_) => {
                match AggProver::new().prove(agg_context) {
                    Ok(()) => {
                        Ok(String::from("SUCCESS"))
                    }
                    Err(_) => {
                        Ok(String::from("BUSY"))
                    }   
                }
            }
            Err(_) => {
                Ok(String::from("BUSY"))
            }
        }
    }

    pub fn final_prove(&mut self, final_context: &FinalContext) -> Result<String> {
        let result = PIPELINE_MUTEX.try_lock();
        match result {
            Ok(_) => {
                match FinalProver::new().prove(final_context) {
                    Ok(()) => {
                        Ok(String::from("SUCCESS"))
                    }
                    Err(_) => {
                        Ok(String::from("BUSY"))
                    }   
                }
            }
            Err(_) => {
                Ok(String::from("BUSY"))
            }
        }
    }

    /// Return prover status
    pub fn get_status(&mut self) -> Result<(String)> {
        let result = PIPELINE_MUTEX.try_lock();
        match result {
            Ok(_) => {
                Ok(String::from("IDLE"))
            }
            Err(_) => {
                Ok(String::from("BUSY"))
            }
        }
    }
}