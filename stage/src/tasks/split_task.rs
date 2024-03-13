
use std::clone::Clone;

#[derive(Debug, Default)]
pub struct SplitTask {
    pub task_id: String,
    pub state: u32,
    pub proof_id: String,
    pub elf_path: String,
    pub seg_path: String,
}

impl Clone for SplitTask {
    fn clone(&self) -> Self {  
        SplitTask {  
            task_id: self.task_id.clone(),
            state: self.state,
            proof_id: self.proof_id.clone(),
            elf_path: self.elf_path.clone(),
            seg_path: self.seg_path.clone(),
        }  
    }  
}