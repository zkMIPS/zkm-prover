mod split_prover;
pub use split_prover::SplitProver;

mod prove_prover;
pub use prove_prover::ProveProver;

mod file_utils;

mod agg_prover;
pub use agg_prover::AggProver;

mod final_prover;
pub use final_prover::FinalProver;

use anyhow::Result;

/// Prover trait
pub trait Prover<T> {
    fn prove(&self, ctx: &T) -> Result<()>;
}