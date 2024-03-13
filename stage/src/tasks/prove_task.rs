

#[derive(Debug, Default)]
pub struct ProveTask {
    pub task_id: String,
    pub state: u32,
    pub proof_id: String,
    pub prove_path: String,
    pub seg_path: String,
}

impl Clone for ProveTask {
    fn clone(&self) -> Self {  
        ProveTask {  
            task_id: self.task_id.clone(),
            state: self.state,
            proof_id: self.proof_id.clone(),
            prove_path: self.prove_path.clone(),
            seg_path: self.seg_path.clone(),
        }  
    }  
}