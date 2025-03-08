use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AggAllContext {
    pub seg_size: u32,
    pub proof_num: u32,
    pub receipt_dir: String,
    pub output_dir: String,
}

impl AggAllContext {
    pub fn new(seg_size: u32, proof_num: u32, receipt_dir: &String, output_dir: &String) -> Self {
        AggAllContext {
            seg_size,
            proof_num,
            receipt_dir: receipt_dir.to_string(),
            output_dir: output_dir.to_string(),
        }
    }
}
