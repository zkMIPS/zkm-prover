use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AggContext {
    pub seg_size: u32,
    pub receipt_path1: Vec<u8>,
    pub receipt_path2: Vec<u8>,
    pub is_agg_1: bool,
    pub is_agg_2: bool,
    pub is_final: bool,
    pub agg_receipt_path: Vec<u8>,
    pub output_dir: String,
}

impl AggContext {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        seg_size: u32,
        receipt_path1: &Vec<u8>,
        receipt_path2: &Vec<u8>,
        is_agg_1: bool,
        is_agg_2: bool,
        is_final: bool,
        agg_receipt_path: &Vec<u8>,
        output_dir: &String,
    ) -> Self {
        AggContext {
            seg_size,
            receipt_path1: receipt_path1.to_owned(),
            receipt_path2: receipt_path2.to_owned(),
            is_agg_1,
            is_agg_2,
            is_final,
            agg_receipt_path: agg_receipt_path.to_owned(),
            output_dir: output_dir.to_string(),
        }
    }
}
