use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AggContext {
    pub basedir: String,
    pub block_no: u64,
    pub seg_size: u32,
    pub proof_path1: String,
    pub proof_path2: String,
    pub pub_value_path1: String,
    pub pub_value_path2: String,
    pub agg_proof_path: String,
    pub agg_pub_value_path: String,
}

impl AggContext {
    pub fn new(
        basedir: &String,
        block_no: u64,
        seg_size: u32,
        proof_path1: &String,
        proof_path2: &String,
        pub_value_path1: &String,
        pub_value_path2: &String,
        agg_proof_path: &String,
        agg_pub_value_path: &String,
    ) -> Self {
        AggContext {
            basedir: basedir.to_string(),
            block_no,
            seg_size,
            proof_path1: proof_path1.to_string(),
            proof_path2: proof_path2.to_string(),
            pub_value_path1: pub_value_path1.to_string(),
            pub_value_path2: pub_value_path2.to_string(),
            agg_proof_path: agg_proof_path.to_string(),
            agg_pub_value_path: agg_pub_value_path.to_string(),
        }
    }
}