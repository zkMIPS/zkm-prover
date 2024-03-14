

#[derive(Debug, Default)]
pub struct AggTask {
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
    pub agg_proof_path: String,
    pub agg_pub_value_path: String,
    pub seg_path: String,
    pub task_seq: usize,
}

impl Clone for AggTask {
    fn clone(&self) -> Self {  
        AggTask {  
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
            agg_proof_path: self.agg_proof_path.clone(),
            agg_pub_value_path: self.agg_pub_value_path.clone(),
            seg_path: self.seg_path.clone(),
            task_seq: self.task_seq,
        }  
    }  
}