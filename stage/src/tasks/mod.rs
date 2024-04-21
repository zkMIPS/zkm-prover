pub const TASK_STATE_INITIAL: u32 = 0;
pub const TASK_STATE_UNPROCESSED: u32 = 1;
pub const TASK_STATE_PROCESSING: u32 = 2;
pub const TASK_STATE_SUCCESS: u32 = 3;
pub const TASK_STATE_FAILED: u32 = 4;

pub const TASK_TIMEOUT: u64 = 7200;

pub mod split_task;
pub use split_task::SplitTask;

pub mod prove_task;
pub use prove_task::ProveTask;

pub mod agg_task;
pub use agg_task::AggAllTask;
pub use agg_task::AggTask;

pub mod final_task;
pub use final_task::FinalTask;

pub const TASK_ITYPE_SPLIT: i32 = 1;
pub const TASK_ITYPE_PROVE: i32 = 2;
pub const TASK_ITYPE_AGG: i32 = 3;
pub const TASK_ITYPE_AGGALL: i32 = 4;
pub const TASK_ITYPE_FINAL: i32 = 5;

pub enum Task {
    Split(SplitTask),
    Prove(ProveTask),
    Agg(AggTask),
    AggAll(AggAllTask),
    Final(FinalTask),
}
