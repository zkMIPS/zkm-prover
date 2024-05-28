use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct GenerateContext {
    pub proof_id: String,
    pub basedir: String,
    pub elf_path: String,
    pub seg_path: String,
    pub prove_path: String,
    pub agg_path: String,
    pub final_path: String,
    pub args: String,
    pub block_no: u64,
    pub seg_size: u32,
}

impl GenerateContext {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        proof_id: &str,
        basedir: &str,
        elf_path: &str,
        seg_path: &str,
        prove_path: &str,
        agg_path: &str,
        final_path: &str,
        args: &str,
        block_no: u64,
        seg_size: u32,
    ) -> Self {
        GenerateContext {
            proof_id: proof_id.to_string(),
            basedir: basedir.to_string(),
            elf_path: elf_path.to_string(),
            seg_path: seg_path.to_string(),
            prove_path: prove_path.to_string(),
            agg_path: agg_path.to_string(),
            final_path: final_path.to_string(),
            args: args.to_string(),
            block_no,
            seg_size,
        }
    }
}
