#[derive(Debug, Default)]
pub struct AggAllTask {
    pub task_id: String,
    pub state: u32,
    pub base_dir: String,
    pub block_no: u64,
    pub seg_size: u32,
    pub proof_num: u32,
    pub proof_id: String,
    pub proof_dir: String,
    pub pub_value_dir: String,
    pub output_dir: String,
}

impl Clone for AggAllTask {
    fn clone(&self) -> Self {
        AggAllTask {
            task_id: self.task_id.clone(),
            state: self.state,
            base_dir: self.base_dir.clone(),
            block_no: self.block_no,
            seg_size: self.seg_size,
            proof_id: self.proof_id.clone(),
            proof_num: self.proof_num,
            proof_dir: self.proof_dir.clone(),
            pub_value_dir: self.pub_value_dir.clone(),
            output_dir: self.output_dir.clone(),
        }
    }
}
