use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SnarkContext {
    pub version: i32,
    pub proof_id: String,
    pub proving_key_path: String,

    pub agg_receipt: Vec<u8>,

    // all belows are temporary variables
    pub common_circuit_data: Vec<u8>,
    pub verifier_only_circuit_data: Vec<u8>,
    pub proof_with_public_inputs: Vec<u8>,
    pub block_public_inputs: Vec<u8>,
}
