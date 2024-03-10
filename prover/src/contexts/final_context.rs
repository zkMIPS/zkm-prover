use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct FinalContext {
    pub basedir: String,
    pub block_no: u64,
    pub seg_size: u32,
    pub proof_path1: String,
    pub proof_path2: String,
    pub pub_value_path1: String,
    pub pub_value_path2: String,
    pub output_dir: String,
}

impl FinalContext {
    pub fn new(
        basedir: &String,
        block_no: u64,
        seg_size: u32,
        proof_path1: &String,
        proof_path2: &String,
        pub_value_path1: &String,
        pub_value_path2: &String,
        output_dir: &String,
    ) -> Self {
        FinalContext {
            basedir: basedir.to_string(),
            block_no,
            seg_size,
            proof_path1: proof_path1.to_string(),
            proof_path2: proof_path2.to_string(),
            pub_value_path1: pub_value_path1.to_string(),
            pub_value_path2: pub_value_path2.to_string(),
            output_dir: output_dir.to_string(),
        }
    }
}