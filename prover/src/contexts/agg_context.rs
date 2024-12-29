use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AggContext {
    pub basedir: String,
    pub block_no: u64,
    pub seg_size: u32,
    pub receipt_path1: String,
    pub receipt_path2: String,
    pub is_agg_1: bool,
    pub is_agg_2: bool,
    pub is_final: bool,
    pub agg_receipt_path: String,
    pub output_dir: String,
}

impl AggContext {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        basedir: &String,
        block_no: u64,
        seg_size: u32,
        receipt_path1: &String,
        receipt_path2: &String,
        is_agg_1: bool,
        is_agg_2: bool,
        is_final: bool,
        agg_receipt_path: &String,
        output_dir: &String,
    ) -> Self {
        AggContext {
            basedir: basedir.to_string(),
            block_no,
            seg_size,
            receipt_path1: receipt_path1.to_string(),
            receipt_path2: receipt_path2.to_string(),
            is_agg_1,
            is_agg_2,
            is_final,
            agg_receipt_path: agg_receipt_path.to_string(),
            output_dir: output_dir.to_string(),
        }
    }
}
