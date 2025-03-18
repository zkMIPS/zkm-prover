use crate::proto::includes::v1::Program;
use crate::stage::tasks::Trace;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ProveTask {
    pub task_id: String,
    pub state: u32,
    pub base_dir: String,

    pub file_no: usize,
    pub done: bool,

    pub segment: Vec<u8>,
    pub program: Program,

    pub output: Vec<Vec<u8>>, // output_receipt
    pub trace: Trace,
}
