#[allow(clippy::module_inception)]
mod executor;

pub use executor::*;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SplitContext {
    pub base_dir: String,
    pub elf_path: String,
    pub block_no: Option<u64>,
    pub seg_size: u32,
    pub seg_path: String,
    pub public_input_path: String,
    pub private_input_path: String,
    // public_values_stream
    pub output_path: String,
    pub args: String,
    pub receipt_inputs_path: String,
}

impl SplitContext {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        basedir: &str,
        _program_id: &str,
        elf_path: &str,
        block_no: Option<u64>,
        seg_size: u32,
        seg_path: &str,
        public_input_path: &str,
        private_input_path: &str,
        output_path: &str,
        args: &str,
        receipt_inputs_path: &str,
    ) -> Self {
        SplitContext {
            base_dir: basedir.to_string(),
            elf_path: elf_path.to_string(),
            block_no,
            seg_size,
            seg_path: seg_path.to_string(),
            public_input_path: public_input_path.to_string(),
            private_input_path: private_input_path.to_string(),
            output_path: output_path.to_string(),
            args: args.to_string(),
            receipt_inputs_path: receipt_inputs_path.to_string(),
        }
    }
}
