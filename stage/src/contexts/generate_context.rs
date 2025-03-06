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
    pub public_input_path: String,
    pub private_input_path: String,
    pub output_stream_path: String,
    pub block_no: u64,
    pub seg_size: u32,
    pub execute_only: bool,
    pub composite_proof: bool,
    pub receipt_inputs_path: String,
    pub receipts_path: String,
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
        public_input_path: &str,
        private_input_path: &str,
        output_stream_path: &str,
        block_no: u64,
        seg_size: u32,
        execute_only: bool,
        composite_proof: bool,
        receipt_inputs_path: &str,
        receipts_path: &str,
    ) -> Self {
        GenerateContext {
            proof_id: proof_id.to_string(),
            basedir: basedir.to_string(),
            elf_path: elf_path.to_string(),
            seg_path: seg_path.to_string(),
            prove_path: prove_path.to_string(),
            agg_path: agg_path.to_string(),
            final_path: final_path.to_string(),
            public_input_path: public_input_path.to_string(),
            private_input_path: private_input_path.to_string(),
            output_stream_path: output_stream_path.to_string(),
            block_no,
            seg_size,
            execute_only,
            composite_proof,
            receipt_inputs_path: receipt_inputs_path.to_string(),
            receipts_path: receipts_path.to_string(),
        }
    }
}
