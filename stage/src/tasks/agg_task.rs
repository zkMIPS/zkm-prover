use crate::tasks::ProveTask;
use crate::tasks::{TASK_STATE_SUCCESS, TASK_STATE_UNPROCESSED};
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct AggAllTask {
    pub task_id: String,
    pub state: u32,
    pub base_dir: String,
    pub block_no: u64,
    pub seg_size: u32,
    pub proof_num: u32,
    pub proof_id: String,
    pub proof_dir: String,
    pub pub_value_dir: String,
    pub output_dir: String,
    pub start_ts: u64,
    pub finish_ts: u64,
    pub node_info: String,
}

impl Clone for AggAllTask {
    fn clone(&self) -> Self {
        AggAllTask {
            task_id: self.task_id.clone(),
            state: self.state,
            base_dir: self.base_dir.clone(),
            block_no: self.block_no,
            seg_size: self.seg_size,
            proof_id: self.proof_id.clone(),
            proof_num: self.proof_num,
            proof_dir: self.proof_dir.clone(),
            pub_value_dir: self.pub_value_dir.clone(),
            output_dir: self.output_dir.clone(),
            start_ts: self.start_ts,
            finish_ts: self.finish_ts,
            node_info: self.node_info.clone(),
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct AggInput {
    pub proof_path: String,
    pub pub_value_path: String,
    pub is_agg: bool,
}

impl AggInput {
    pub fn from_prove_task(prove_task: &ProveTask) -> AggInput {
        AggInput {
            proof_path: prove_task.prove_path.clone(),
            pub_value_path: prove_task.pub_value_path.clone(),
            is_agg: false,
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct AggTask {
    pub file_key: String,
    pub task_id: String,
    pub state: u32,
    pub base_dir: String,
    pub block_no: u64,
    pub seg_size: u32,
    pub proof_id: String,
    pub input1: AggInput,
    pub input2: AggInput,
    pub is_final: bool,
    pub from_prove: bool,
    pub output_proof_path: String,
    pub output_pub_value_path: String,
    pub output_dir: String,
    pub start_ts: u64,
    pub finish_ts: u64,
    pub node_info: String,

    // depend
    pub left: Option<String>,
    pub right: Option<String>,
}

impl Clone for AggTask {
    fn clone(&self) -> Self {
        AggTask {
            file_key: self.file_key.clone(),
            task_id: self.task_id.clone(),
            state: self.state,
            base_dir: self.base_dir.clone(),
            block_no: self.block_no,
            seg_size: self.seg_size,
            proof_id: self.proof_id.clone(),
            input1: self.input1.clone(),
            input2: self.input2.clone(),
            is_final: self.is_final,
            from_prove: self.from_prove,
            output_proof_path: self.output_proof_path.clone(),
            output_pub_value_path: self.output_pub_value_path.clone(),
            output_dir: self.output_dir.clone(),
            start_ts: self.start_ts,
            finish_ts: self.finish_ts,
            node_info: self.node_info.clone(),
            left: self.left.clone(),
            right: self.right.clone(),
        }
    }
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

    pub fn set_out_path(&mut self, prove_dir: &str) {
        self.output_proof_path = format!("{}/proof/{}", prove_dir, self.file_key);
        self.output_pub_value_path = format!("{}/pub_value/{}", prove_dir, self.file_key);
    }

    pub fn to_agg_input(&self) -> AggInput {
        AggInput {
            proof_path: self.output_proof_path.clone(),
            pub_value_path: self.output_pub_value_path.clone(),
            is_agg: !self.from_prove,
        }
    }

    pub fn init_from_single_prove_task(prove_task: &ProveTask, prove_dir: &str) -> AggTask {
        let mut agg_task = AggTask {
            file_key: format!("{}", prove_task.file_no),
            task_id: uuid::Uuid::new_v4().to_string(),
            base_dir: prove_task.base_dir.clone(),
            block_no: prove_task.block_no,
            state: TASK_STATE_SUCCESS,
            seg_size: prove_task.seg_size,
            proof_id: prove_task.proof_id.clone(),
            from_prove: true,
            ..Default::default()
        };
        agg_task.set_out_path(prove_dir);
        agg_task
    }

    pub fn init_from_two_prove_task(
        left: &ProveTask,
        right: &ProveTask,
        prove_dir: &str,
        agg_index: i32,
    ) -> AggTask {
        let mut agg_task = AggTask {
            // file_key: format!("{}-{}", left.file_no, right.file_no),
            file_key: format!("agg{}", agg_index),
            task_id: uuid::Uuid::new_v4().to_string(),
            base_dir: left.base_dir.clone(),
            block_no: left.block_no,
            state: TASK_STATE_UNPROCESSED,
            seg_size: left.seg_size,
            proof_id: left.proof_id.clone(),
            input1: AggInput::from_prove_task(left),
            input2: AggInput::from_prove_task(right),
            ..Default::default()
        };
        agg_task.set_out_path(prove_dir);
        agg_task
    }

    pub fn init_from_two_agg_task(
        left: &AggTask,
        right: &AggTask,
        prove_dir: &str,
        agg_index: i32,
    ) -> AggTask {
        let mut agg_task = AggTask {
            // file_key: format!("{}-{}", left.file_key, right.file_key),
            file_key: format!("agg{}", agg_index),
            task_id: uuid::Uuid::new_v4().to_string(),
            base_dir: left.base_dir.clone(),
            block_no: left.block_no,
            state: TASK_STATE_UNPROCESSED,
            seg_size: left.seg_size,
            proof_id: left.proof_id.clone(),
            input1: left.to_agg_input(),
            input2: right.to_agg_input(),
            ..Default::default()
        };
        if !left.from_prove {
            agg_task.left = Some(left.task_id.clone());
        }
        if !right.from_prove {
            agg_task.right = Some(right.task_id.clone());
        }
        agg_task.set_out_path(prove_dir);
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
        let agg_task = crate::tasks::AggTask::init_from_single_prove_task(&prove_task, "/test");
        assert!(agg_task.state == TASK_STATE_SUCCESS);
        assert!(agg_task.file_key == format!("{}", prove_task.file_no));
    }

    #[test]
    fn test_init_from_two_prove_task() {
        let left_prove_task = ProveTask {
            file_no: 1,
            ..Default::default()
        };
        let right_prove_task = ProveTask {
            file_no: 2,
            ..Default::default()
        };
        let agg_task = crate::tasks::AggTask::init_from_two_prove_task(
            &left_prove_task,
            &right_prove_task,
            "/test",
            1,
        );
        assert!(agg_task.state == TASK_STATE_UNPROCESSED);
        assert!(agg_task.file_key == format!("agg{}", 1));
    }

    #[test]
    fn test_init_from_two_agg_task() {
        let left_agg_task = AggTask {
            file_key: "1".to_string(),
            ..Default::default()
        };
        let right_agg_task = AggTask {
            file_key: "2".to_string(),
            ..Default::default()
        };
        let agg_task = crate::tasks::AggTask::init_from_two_agg_task(
            &left_agg_task,
            &right_agg_task,
            "/test",
            1,
        );
        assert!(agg_task.state == TASK_STATE_UNPROCESSED);
        assert!(agg_task.file_key == format!("agg{}", 1));
        assert!(agg_task.left.is_some());
        assert!(agg_task.right.is_some());
    }
}
