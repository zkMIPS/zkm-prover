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
    pub is_agg_1: bool,
    pub is_agg_2: bool,
    pub is_final: bool,
    pub agg_proof_path: String,
    pub agg_pub_value_path: String,
    pub output_dir: String,
}

impl AggContext {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        basedir: &String,
        block_no: u64,
        seg_size: u32,
        proof_path1: &String,
        proof_path2: &String,
        pub_value_path1: &String,
        pub_value_path2: &String,
        is_agg_1: bool,
        is_agg_2: bool,
        is_final: bool,
        agg_proof_path: &String,
        agg_pub_value_path: &String,
        output_dir: &String,
    ) -> Self {
        AggContext {
            basedir: basedir.to_string(),
            block_no,
            seg_size,
            proof_path1: proof_path1.to_string(),
            proof_path2: proof_path2.to_string(),
            pub_value_path1: pub_value_path1.to_string(),
            pub_value_path2: pub_value_path2.to_string(),
            is_agg_1,
            is_agg_2,
            is_final,
            agg_proof_path: agg_proof_path.to_string(),
            agg_pub_value_path: agg_pub_value_path.to_string(),
            output_dir: output_dir.to_string(),
        }
    }
}
