use crate::stage::tasks::{ProveTask, Trace, TASK_STATE_SUCCESS, TASK_STATE_UNPROCESSED};
use serde::Deserialize;
use serde::Serialize;

use crate::proto::includes::v1::AggregateInput;
pub fn from_prove_task(prove_task: &ProveTask, index: usize) -> AggregateInput {
    assert!(prove_task.output.len() > index);
    AggregateInput {
        // we put the receipt of prove_task, instead of the file path
        receipt_input: prove_task.output[index].clone(),
        computed_request_id: prove_task.task_id.clone(),
        is_agg: false,
    }
}
//}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AggTask {
    pub task_id: String,
    pub state: u32,

    pub block_no: Option<u64>,
    pub seg_size: u32,
    pub proof_id: String,
    pub input1: AggregateInput,
    pub input2: AggregateInput,
    pub is_final: bool,
    pub from_prove: bool,
    pub agg_index: i32,

    pub trace: Trace,
    pub output: Vec<u8>, // output_receipt: Vec<u8>,

    // depend
    pub left: Option<String>,
    pub right: Option<String>,
}

impl AggTask {
    pub fn clear_child_task(&mut self, task_id: &str) -> bool {
        if self.state == TASK_STATE_UNPROCESSED {
            if let Some(left) = &self.left {
                if *left == task_id {
                    self.left = None;
                    return true;
                }
            }
            if let Some(right) = &self.right {
                if *right == task_id {
                    self.right = None;
                    return true;
                }
            }
        }
        false
    }

    pub fn to_agg_input(&self) -> AggregateInput {
        // For the receipt_input, nothing we can do here when we create the calculation graph, so give it a default value
        AggregateInput {
            receipt_input: vec![],
            computed_request_id: self.task_id.clone(),
            is_agg: !self.from_prove,
        }
    }

    // FIXME: if we have a single prove task, and try to aggegate its root proof, panic will raise
    // So we just set up the state successful
    pub fn init_from_single_prove_task(prove_task: &ProveTask, agg_index: i32) -> AggTask {
        let task_id = uuid::Uuid::new_v4().to_string();
        AggTask {
            task_id: task_id.clone(),
            block_no: prove_task.program.block_no,
            state: TASK_STATE_SUCCESS,
            seg_size: prove_task.program.seg_size,
            proof_id: prove_task.program.proof_id.clone(),
            from_prove: true,
            agg_index,
            ..Default::default()
        }
    }


    pub fn init_from_two_prove_task(
        left: &ProveTask,
        right: &ProveTask,
        agg_index: i32,
    ) -> AggTask {
        AggTask {
            task_id: uuid::Uuid::new_v4().to_string(),
            block_no: left.program.block_no,
            state: TASK_STATE_UNPROCESSED,
            seg_size: left.program.seg_size,
            proof_id: left.program.proof_id.clone(),
            input1: from_prove_task(left),
            input2: from_prove_task(right),
            agg_index,
            ..Default::default()
        }
    }

    pub fn init_from_two_agg_task(left: &AggTask, right: &AggTask, agg_index: i32) -> AggTask {
        let mut agg_task = AggTask {
            task_id: uuid::Uuid::new_v4().to_string(),
            block_no: left.block_no,
            state: TASK_STATE_UNPROCESSED,
            seg_size: left.seg_size,
            proof_id: left.proof_id.clone(),
            input1: left.to_agg_input(),
            input2: right.to_agg_input(),
            agg_index,
            ..Default::default()
        };
        if !left.from_prove {
            agg_task.left = Some(left.task_id.clone());
        }
        if !right.from_prove {
            agg_task.right = Some(right.task_id.clone());
        }
        agg_task
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_clear_child_task() {
        let left_task_id = "test_id_1";
        let right_task_id = "test_id_2";
        let mut agg_task = AggTask {
            state: TASK_STATE_UNPROCESSED,
            left: Some(left_task_id.to_string()),
            right: Some(right_task_id.to_string()),
            ..Default::default()
        };
        agg_task.clear_child_task(left_task_id);
        agg_task.clear_child_task(right_task_id);
        assert!(agg_task.left.is_none());
        assert!(agg_task.right.is_none());
    }

    #[test]
    fn test_init_from_single_prove_task() {
        let prove_task = ProveTask {
            file_no: 1,
            ..Default::default()
        };
        let agg_task = crate::stage::tasks::AggTask::init_from_single_prove_task(&prove_task, 1);
        assert!(agg_task.state == TASK_STATE_SUCCESS);
    }

    #[test]
    fn test_init_from_two_prove_task() {
        let left_prove_task = ProveTask {
            file_no: 1,
            output: vec![1, 2, 3],
            ..Default::default()
        };
        let right_prove_task = ProveTask {
            file_no: 2,
            output: vec![3, 4, 5],
            ..Default::default()
        };
        let agg_task = crate::stage::tasks::AggTask::init_from_two_prove_task(
            &left_prove_task,
            &right_prove_task,
            1,
        );
        assert!(agg_task.state == TASK_STATE_UNPROCESSED);
    }

    #[test]
    fn test_init_from_two_agg_task() {
        let left_agg_task = AggTask {
            task_id: "1".to_string(),
            ..Default::default()
        };
        let right_agg_task = AggTask {
            task_id: "2".to_string(),
            ..Default::default()
        };
        let agg_task = crate::stage::tasks::AggTask::init_from_two_agg_task(
            &left_agg_task,
            &right_agg_task,
            1,
        );
        assert!(agg_task.state == TASK_STATE_UNPROCESSED);
        assert!(agg_task.left.is_some());
        assert!(agg_task.right.is_some());
    }
}
