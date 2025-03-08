use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ProveContext {
    pub block_no: Option<u64>,
    pub seg_size: u32,
    pub segment: Vec<u8>,
    pub receipt_output: String,
    pub receipts_input: Vec<Vec<u8>>,
}

impl ProveContext {
    pub fn new(
        block_no: Option<u64>,
        seg_size: u32,
        segment: &[u8],
        receipts_input: &Vec<Vec<u8>>,
    ) -> Self {
        ProveContext {
            block_no,
            seg_size,
            segment: segment.to_owned(),
            receipts_input: receipts_input.to_owned(),
            ..Default::default()
        }
    }
}
