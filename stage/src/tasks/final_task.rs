#[derive(Debug, Default)]
pub struct FinalTask {
    pub task_id: String,
    pub state: u32,
    pub proof_id: String,
    pub input_dir: String,
    pub output_path: String,
    pub start_ts: u64,
    pub finish_ts: u64,
}

impl Clone for FinalTask {
    fn clone(&self) -> Self {
        FinalTask {
            task_id: self.task_id.clone(),
            state: self.state,
            proof_id: self.proof_id.clone(),
            input_dir: self.input_dir.clone(),
            output_path: self.output_path.clone(),
            start_ts: self.start_ts,
            finish_ts: self.finish_ts,
        }
    }
}
