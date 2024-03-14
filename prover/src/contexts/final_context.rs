use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct FinalContext {
    pub basedir: String,
    pub block_no: u64,
    pub seg_size: u32,
    pub proof_path: String,
    pub pub_value_path: String,
    pub output_dir: String,
}

impl FinalContext {
    pub fn new(
        basedir: &String,
        block_no: u64,
        seg_size: u32,
        proof_path: &String,
        pub_value_path: &String,
        output_dir: &String,
    ) -> Self {
        FinalContext {
            basedir: basedir.to_string(),
            block_no,
            seg_size,
            proof_path: proof_path.to_string(),
            pub_value_path: pub_value_path.to_string(),
            output_dir: output_dir.to_string(),
        }
    }
}