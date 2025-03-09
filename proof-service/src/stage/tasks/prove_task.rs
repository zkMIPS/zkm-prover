use crate::proto::includes::v1::Program;
use crate::stage::tasks::Trace;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ProveTask {
    pub task_id: String,
    pub state: u32,
    pub base_dir: String,

    pub file_no: usize,

    pub segment: Vec<u8>,
    pub program: Program,

    //pub block_no: u64,
    //pub seg_size: u32,
    //pub proof_id: String,
    pub receipt_output: Vec<u8>,
    //pub receipts_input: Vec<Vec<u8>>,
    //pub seg_path: String,
    pub trace: Trace,
}
