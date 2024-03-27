mod root_prover;
pub use root_prover::RootProver;

mod file_utils;
pub use file_utils::read_file_bin;
pub use file_utils::read_file_content;

mod agg_prover;
pub use agg_prover::AggProver;

mod agg_all_prover;
pub use agg_all_prover::AggAllProver;

use anyhow::Result;

/// Prover trait
pub trait Prover<T> {
    fn prove(&self, ctx: &T) -> Result<()>;
}
