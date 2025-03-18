use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ProveContext {
    pub proof_id: String,
    // pub block_no: Option<u64>,
    pub index: usize,
    pub done: bool,
    pub elf: Vec<u8>,
    pub segment: Vec<u8>,
    // pub receipts_input: Vec<Vec<u8>>,
}
