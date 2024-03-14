

#[derive(Debug, Default)]
pub struct FinalTask {
    pub task_id: String,
    pub state: u32,
    pub base_dir: String,
    pub block_no: u64,
    pub seg_size: u32,
    pub proof_id: String,
    pub proof_path: String,
    pub pub_value_path: String,
    pub out_path: String,
}

impl Clone for FinalTask {
    fn clone(&self) -> Self {  
        FinalTask {  
            task_id: self.task_id.clone(),
            state: self.state,
            base_dir: self.base_dir.clone(),
            block_no: self.block_no,
            seg_size: self.seg_size,
            proof_id: self.proof_id.clone(),
            proof_path: self.proof_path.clone(),
            pub_value_path: self.pub_value_path.clone(),
            out_path: self.out_path.clone(),
        }  
    }  
}