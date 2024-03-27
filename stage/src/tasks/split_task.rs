#[derive(Debug, Default)]
pub struct SplitTask {
    pub task_id: String,
    pub state: u32,
    pub proof_id: String,
    pub base_dir: String,
    pub elf_path: String,
    pub seg_path: String,
    pub block_no: u64,
    pub seg_size: u32,
}

impl Clone for SplitTask {
    fn clone(&self) -> Self {
        SplitTask {
            task_id: self.task_id.clone(),
            state: self.state,
            proof_id: self.proof_id.clone(),
            base_dir: self.base_dir.clone(),
            elf_path: self.elf_path.clone(),
            seg_path: self.seg_path.clone(),
            block_no: self.block_no,
            seg_size: self.seg_size,
        }
    }
}
