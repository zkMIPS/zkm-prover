use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ProveContext {
    pub basedir: String,
    pub block_no: u64,
    pub seg_size: u32,
    pub seg_path: String,
    pub proof_path: String,
    pub pub_value_path: String,
}

impl ProveContext {
    pub fn new(
        basedir: &String,
        block_no: u64,
        seg_size: u32,
        seg_path: &String,
        proof_path: &String,
        pub_value_path: &String,
    ) -> Self {
        ProveContext {
            basedir: basedir.to_string(),
            block_no,
            seg_size,
            seg_path: seg_path.to_string(),
            proof_path: proof_path.to_string(),
            pub_value_path: pub_value_path.to_string(),
        }
    }
}
