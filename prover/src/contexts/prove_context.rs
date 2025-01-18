use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ProveContext {
    pub basedir: String,
    pub block_no: u64,
    pub seg_size: u32,
    pub seg_path: String,
    pub receipt_path: String,
    pub receipts_path: String,
}

impl ProveContext {
    pub fn new(
        basedir: &String,
        block_no: u64,
        seg_size: u32,
        seg_path: &String,
        receipt_path: &String,
        receipts_path: &String,
    ) -> Self {
        ProveContext {
            basedir: basedir.to_string(),
            block_no,
            seg_size,
            seg_path: seg_path.to_string(),
            receipt_path: receipt_path.to_string(),
            receipts_path: receipts_path.to_string(),
        }
    }
}
