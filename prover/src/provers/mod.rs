mod root_prover;
pub use root_prover::RootProver;

mod agg_prover;
pub use agg_prover::AggProver;

mod agg_all_prover;
mod snark_prover;

pub use agg_all_prover::AggAllProver;

use anyhow::Result;
use lazy_static::lazy_static;
use std::collections::HashMap;

use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::plonk::config::PoseidonGoldilocksConfig;
use zkm_prover::all_stark::AllStark;
use zkm_prover::config::StarkConfig;
use zkm_prover::fixed_recursive_verifier::AllRecursiveCircuits;

use once_cell::sync::OnceCell;
use std::sync::Mutex;
pub use zkm_recursion::DEGREE_BITS_RANGE;


pub const MIN_SEG_SIZE: usize = 1 << 16;
pub const MAX_SEG_SIZE: usize = 1 << 22;

pub fn valid_seg_size(seg_size: usize) -> bool {
    if (MIN_SEG_SIZE..=MAX_SEG_SIZE).contains(&seg_size) {
        return true;
    }
    false
}


pub trait Prover<T> {
    fn prove(&self, ctx: &mut T) -> Result<()>;
}
type F = GoldilocksField;
const D: usize = 2;
type C = PoseidonGoldilocksConfig;

static INSTANCE_ALL_CIRCUITS: OnceCell<Mutex<AllRecursiveCircuits<F, C, D>>> = OnceCell::new();

pub fn instance() -> &'static Mutex<AllRecursiveCircuits<F, C, D>> {
    INSTANCE_ALL_CIRCUITS.get_or_init(|| {
        let all_stark = AllStark::<F, D>::default();
        let config = StarkConfig::standard_fast_config();
        // Preprocess all circuits.
        Mutex::new(AllRecursiveCircuits::<F, C, D>::new(
            &all_stark,
            &DEGREE_BITS_RANGE,
            &config,
        ))
    })
}
