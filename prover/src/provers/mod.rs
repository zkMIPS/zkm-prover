mod root_prover;
pub use root_prover::RootProver;

mod agg_prover;
pub use agg_prover::AggProver;

mod agg_all_prover;
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

const MIN_SEG_SIZE: usize = 65536;
const MAX_SEG_SIZE: usize = 262144;
/// Prover trait
pub trait Prover<T> {
    fn prove(&self, ctx: &T) -> Result<()>;
}

const DEGREE_BITS_RANGE: [[std::ops::Range<usize>; 6]; 1] =
    [[10..21, 12..22, 12..21, 8..21, 8..21, 13..23]];
// const DEGREE_BITS_RANGE: [[std::ops::Range<usize>; 6]; 5] = [
//     [16..17, 12..13, 10..16, 9..12, 15..17, 17..19],
//     [16..17, 15..17, 12..19, 9..14, 15..17, 19..20],
//     [16..17, 15..17, 12..19, 11..14, 16..19, 19..21],
//     [16..17, 17..18, 14..19, 11..14, 16..19, 21..22],
//     [16..18, 16..20, 16..21, 14..15, 18..21, 21..23],
// ];

lazy_static! {
    static ref SEG_SIZE_TO_BITS: HashMap<usize, usize> = {
        let mut map = HashMap::new();
        map.insert(262144, 0);
        map
    };
}

fn select_degree_bits(seg_size: usize) -> [std::ops::Range<usize>; 6] {
    match SEG_SIZE_TO_BITS.get(&seg_size) {
        Some(s) => DEGREE_BITS_RANGE[*s].clone(),
        None => panic!(
            "Invalid segment size, supported: {:?}",
            SEG_SIZE_TO_BITS.keys()
        ),
    }
}

pub fn valid_seg_size(seg_size: usize) -> bool {
    if (MIN_SEG_SIZE..=MAX_SEG_SIZE).contains(&seg_size) {
        return true;
    }
    false
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
            &select_degree_bits(MAX_SEG_SIZE),
            &config,
        ))
    })
}
