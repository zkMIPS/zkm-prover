mod root_prover;
pub use root_prover::RootProver;

mod agg_prover;
pub use agg_prover::AggProver;

mod agg_all_prover;
pub use agg_all_prover::AggAllProver;

use anyhow::Result;
use lazy_static::lazy_static;
use std::collections::HashMap;

/// Prover trait
pub trait Prover<T> {
    fn prove(&self, ctx: &T) -> impl std::future::Future<Output = Result<()>> + Send;
}

const DEGREE_BITS_RANGE: [[std::ops::Range<usize>; 6]; 5] = [
    [10..21, 10..15, 10..18, 8..15, 10..21, 15..23],
    [10..21, 12..22, 13..21, 8..21, 10..21, 13..23],
    [10..21, 12..22, 13..21, 8..21, 10..21, 13..23],
    [10..21, 12..22, 13..21, 8..21, 10..21, 13..23],
    [10..21, 12..22, 13..21, 8..21, 10..21, 13..23],
];
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
        map.insert(1024, 0);
        map.insert(16384, 1);
        map.insert(32768, 2);
        map.insert(65536, 3);
        map.insert(262144, 4);
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
    if SEG_SIZE_TO_BITS.contains_key(&seg_size) {
        return true;
    }
    false
}
