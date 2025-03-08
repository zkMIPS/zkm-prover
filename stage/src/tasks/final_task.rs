use crate::tasks::Trace;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FinalTask {
    pub task_id: String,
    pub state: u32,
    pub proof_id: String,

    pub input_dir: String,
    pub output_path: String,

    pub trace: Trace,
}
