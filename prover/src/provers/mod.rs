mod root_prover;
pub use root_prover::RootProver;

mod agg_prover;
pub use agg_prover::AggProver;

mod agg_all_prover;
mod snark_prover;
pub use agg_all_prover::AggAllProver;
pub use snark_prover::SnarkProver;

use anyhow::Result;

use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::plonk::config::PoseidonGoldilocksConfig;
use zkm_prover::fixed_recursive_verifier::AllRecursiveCircuits;

use once_cell::sync::OnceCell;
use std::sync::Mutex;

pub const MIN_SEG_SIZE: usize = 1 << 10;
pub const MAX_SEG_SIZE: usize = 1 << 22;

pub fn valid_seg_size(seg_size: usize) -> bool {
    if (MIN_SEG_SIZE..=MAX_SEG_SIZE).contains(&seg_size) {
        return true;
    }
    false
}

pub trait Prover<T, R> {
    fn prove(&self, ctx: &T) -> Result<(bool, R)>;
}
type F = GoldilocksField;
const D: usize = 2;
type C = PoseidonGoldilocksConfig;

static INSTANCE_ALL_CIRCUITS: OnceCell<Mutex<AllRecursiveCircuits<F, C, D>>> = OnceCell::new();

pub fn instance() -> &'static Mutex<AllRecursiveCircuits<F, C, D>> {
    // FIXME: replace it by zkm_recursion::create_recursive_circuit()
    INSTANCE_ALL_CIRCUITS.get_or_init(|| Mutex::new(zkm_recursion::create_recursive_circuit()))
}
