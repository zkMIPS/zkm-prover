use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ProveTask {
    pub file_no: usize,
    pub task_id: String,
    pub state: u32,
    pub base_dir: String,
    pub block_no: u64,
    pub seg_size: u32,
    pub proof_id: String,
    pub prove_path: String,
    pub pub_value_path: String,
    pub seg_path: String,
    pub start_ts: u64,
    pub finish_ts: u64,
    pub node_info: String,
}

impl Clone for ProveTask {
    fn clone(&self) -> Self {
        ProveTask {
            file_no: self.file_no,
            task_id: self.task_id.clone(),
            state: self.state,
            base_dir: self.base_dir.clone(),
            block_no: self.block_no,
            seg_size: self.seg_size,
            proof_id: self.proof_id.clone(),
            prove_path: self.prove_path.clone(),
            pub_value_path: self.pub_value_path.clone(),
            seg_path: self.seg_path.clone(),
            start_ts: self.start_ts,
            finish_ts: self.finish_ts,
            node_info: self.node_info.clone(),
        }
    }
}
