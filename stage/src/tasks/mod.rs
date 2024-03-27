pub static TASK_STATE_INITIAL: u32 = 0;
pub static TASK_STATE_UNPROCESSED: u32 = 1;
pub static TASK_STATE_PROCESSING: u32 = 2;
pub static TASK_STATE_SUCCESS: u32 = 3;
pub static TASK_STATE_FAILED: u32 = 4;

pub static TASK_TIMEOUT: u64 = 1800;

pub mod split_task;
pub use split_task::SplitTask;

pub mod prove_task;
pub use prove_task::ProveTask;

pub mod agg_task;
pub use agg_task::AggAllTask;

pub mod final_task;
pub use final_task::FinalTask;

pub enum Task {
    Split(SplitTask),
    Prove(ProveTask),
    Agg(AggAllTask),
    Final(FinalTask),
}
