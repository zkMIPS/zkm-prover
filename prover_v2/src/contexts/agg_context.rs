use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AggContext {
    pub index: usize,
    pub zkm_circuit_witness: Vec<u8>,
    pub is_agg_1: bool,
    pub is_agg_2: bool,
    pub is_final: bool,
}
