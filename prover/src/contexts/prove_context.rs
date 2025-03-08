use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ProveContext {
    pub block_no: Option<u64>,
    pub seg_size: u32,
    pub segment: Vec<u8>,
    pub receipt_path: String,
    pub receipts_path: String,
}

impl ProveContext {
    pub fn new(
        block_no: Option<u64>,
        seg_size: u32,
        segment: &[u8],
        receipt_path: &String,
        receipts_path: &String,
    ) -> Self {
        ProveContext {
            block_no,
            seg_size,
            segment: segment.to_owned(),
            receipt_path: receipt_path.to_string(),
            receipts_path: receipts_path.to_string(),
        }
    }
}
