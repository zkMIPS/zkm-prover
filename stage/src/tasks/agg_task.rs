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

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct AggTask {
    pub file_key: String,
    pub task_id: String,
    pub state: u32,
    pub base_dir: String,
    pub block_no: u64,
    pub seg_size: u32,
    pub proof_id: String,
    pub proof_path1: String,
    pub proof_path2: String,
    pub pub_value_path1: String,
    pub pub_value_path2: String,
    pub is_agg1: bool,
    pub is_agg2: bool,
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
            proof_path1: self.proof_path1.clone(),
            proof_path2: self.proof_path2.clone(),
            pub_value_path1: self.pub_value_path1.clone(),
            pub_value_path2: self.pub_value_path2.clone(),
            is_agg1: self.is_agg1,
            is_agg2: self.is_agg2,
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
                    return false;
                }
            }
        }
        false
    }

    pub fn set_out_path(&mut self, prove_dir: &str) {
        self.output_proof_path = format!("{}/proof/{}", prove_dir, self.file_key);
        self.output_pub_value_path = format!("{}/pub_value/{}", prove_dir, self.file_key);
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
        left_is_agg: bool,
        right_is_agg: bool,
        prove_dir: &str,
    ) -> AggTask {
        let mut agg_task = AggTask {
            file_key: format!("{}-{}", left.file_no, right.file_no),
            task_id: uuid::Uuid::new_v4().to_string(),
            base_dir: left.base_dir.clone(),
            block_no: left.block_no,
            state: TASK_STATE_UNPROCESSED,
            seg_size: left.seg_size,
            proof_id: left.proof_id.clone(),
            proof_path1: left.prove_path.clone(),
            proof_path2: right.prove_path.clone(),
            pub_value_path1: left.pub_value_path.clone(),
            pub_value_path2: right.pub_value_path.clone(),
            is_agg1: left_is_agg,
            is_agg2: right_is_agg,
            ..Default::default()
        };
        agg_task.set_out_path(prove_dir);
        agg_task
    }

    pub fn init_from_two_agg_task(left: &AggTask, right: &AggTask, prove_dir: &str) -> AggTask {
        let mut agg_task = AggTask {
            file_key: format!("{}-{}", left.file_key, right.file_key),
            task_id: uuid::Uuid::new_v4().to_string(),
            base_dir: left.base_dir.clone(),
            block_no: left.block_no,
            state: TASK_STATE_UNPROCESSED,
            seg_size: left.seg_size,
            proof_id: left.proof_id.clone(),
            proof_path1: left.output_proof_path.clone(),
            proof_path2: right.output_proof_path.clone(),
            pub_value_path1: left.output_pub_value_path.clone(),
            pub_value_path2: right.output_pub_value_path.clone(),
            is_agg1: !left.from_prove,
            is_agg2: !right.from_prove,
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
