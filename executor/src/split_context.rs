use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SplitContext {
    pub basedir: String,
    pub elf_path: String,
    pub block_no: u64,
    pub seg_size: u32,
    pub seg_path: String,
    pub args: String,
}

impl SplitContext {
    pub fn new(
        basedir: &str,
        elf_path: &str,
        block_no: u64,
        seg_size: u32,
        seg_path: &String,
        args: &str,
    ) -> Self {
        SplitContext {
            basedir: basedir.to_string(),
            elf_path: elf_path.to_string(),
            block_no,
            seg_size,
            seg_path: seg_path.to_string(),
            args: args.to_string(),
        }
    }
}
