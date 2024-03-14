

#[derive(Debug, Default)]
pub struct FinalTask {
    pub task_id: String,
    pub state: u32,
    pub base_dir: String,
    pub block_no: u64,
    pub seg_size: u32,
    pub proof_id: String,
    pub proof_path1: String,
    pub pub_value_path1: String,
    pub proof_path2: String,
    pub pub_value_path2: String,
    pub is_agg_1: bool,
    pub is_agg_2: bool,
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
            proof_path1: self.proof_path1.clone(),
            pub_value_path1: self.pub_value_path1.clone(),
            proof_path2: self.proof_path2.clone(),
            pub_value_path2: self.pub_value_path2.clone(),
            is_agg_1: self.is_agg_1,
            is_agg_2: self.is_agg_2,
            out_path: self.out_path.clone(),
        }  
    }  
}