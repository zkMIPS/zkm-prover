use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AggAllContext {
    pub basedir: String,
    pub block_no: u64,
    pub seg_size: u32,
    pub proof_num: u32,
    pub proof_dir: String,
    pub pub_value_dir: String,
    pub output_dir: String,
}

impl AggAllContext {
    pub fn new(
        basedir: &String,
        block_no: u64,
        seg_size: u32,
        proof_num: u32,
        proof_dir: &String,
        pub_value_dir: &String,
        output_dir: &String,
    ) -> Self {
        AggAllContext {
            basedir: basedir.to_string(),
            block_no,
            seg_size,
            proof_num,
            proof_dir: proof_dir.to_string(),
            pub_value_dir: pub_value_dir.to_string(),
            output_dir: output_dir.to_string(),
        }
    }
}
