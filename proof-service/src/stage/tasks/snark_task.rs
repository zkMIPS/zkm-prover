use crate::stage::tasks::Trace;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SnarkTask {
    pub task_id: String,
    pub state: u32,
    pub proof_id: String,
    pub version: i32,

    pub input_dir: String,
    pub output_path: String,

    pub trace: Trace,
}
