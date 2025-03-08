use crate::program::v1::Program;
use crate::tasks::Trace;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ProveTask {
    pub task_id: String,
    pub state: u32,
    pub base_dir: String,

    pub program: Program,
    pub file_no: usize,
    pub segment: Vec<u8>,

    //pub block_no: u64,
    pub seg_size: u32,
    pub proof_id: String,
    pub receipt_path: String,
    pub receipts_path: String,
    //pub seg_path: String,
    pub trace: Trace,
}
