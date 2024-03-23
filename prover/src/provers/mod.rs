
mod root_prover;
pub use root_prover::RootProver;

mod file_utils;
pub use file_utils::read_file_content;
pub use file_utils::read_file_bin;

mod agg_prover;
pub use agg_prover::AggProver;

mod agg_all_prover;
pub use agg_all_prover::AggAllProver;

use anyhow::Result;

/// Prover trait
pub trait Prover<T> {
    fn prove(&self, ctx: &T) -> Result<()>;
}

const DEGREE_BITS_RANGE: [[std::ops::Range<usize>; 6]; 5] = [
    [16..17, 12..13, 10..16, 9..12, 15..17, 17..19],
    [16..17, 15..17, 12..19, 9..14, 15..17, 19..20],
    [16..17, 15..17, 12..19, 11..14, 16..19, 19..21],
    [16..17, 17..18, 14..19, 11..14, 16..19, 21..22],
    [16..18, 16..20, 16..21, 14..15, 18..21, 21..23],
];

fn select_degree_bits(seg_size: usize) -> [std::ops::Range<usize>; 6] {
    let seg_size_to_bits = std::collections::BTreeMap::from([
        (1024, 0),
        (16384, 1),
        (32768, 2),
        (65536, 3),
        (262144, 4),
    ]);
    match seg_size_to_bits.get(&seg_size) {
        Some(s) => DEGREE_BITS_RANGE[*s].clone(),
        None => panic!(
            "Invalid segment size, supported: {:?}",
            seg_size_to_bits.keys()
        ),
    }
}

pub fn valid_seg_size(seg_size: usize) -> bool {
    if seg_size == 1024 || 
        seg_size == 16384 ||
        seg_size == 32768 ||
        seg_size == 65536 ||
        seg_size == 262144  {
        return true;
    }
    false
}