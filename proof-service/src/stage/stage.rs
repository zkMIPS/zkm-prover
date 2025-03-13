use crate::stage::{
    safe_read,
    tasks::{
        agg_task::{self, AggTask},
        {ProveTask, SnarkTask, SplitTask}, {Trace, TASK_STATE_PROCESSING},
        {TASK_STATE_FAILED, TASK_STATE_INITIAL, TASK_STATE_SUCCESS, TASK_STATE_UNPROCESSED},
    },
};
use std::fmt::{Debug, Formatter};

use crate::proto::stage_service::v1::Step;
use crate::stage::tasks::generate_task::GenerateTask;
use common::file;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_timestamp() -> u64 {
    let now = SystemTime::now();
    let duration_since_epoch = now.duration_since(UNIX_EPOCH).unwrap();
    duration_since_epoch.as_secs()
}
#[derive(Default)]
pub struct Stage {
    pub generate_task: GenerateTask,
    pub split_task: SplitTask,
    pub prove_tasks: Vec<ProveTask>,
    pub agg_tasks: Vec<AggTask>,
    pub snark_task: SnarkTask,
    pub is_error: bool,
    pub errmsg: String,
    pub step: Step,
}

macro_rules! on_task {
    ($src:ident, $dst:ident, $stage:ident) => {
        //assert!($src.proof_id == $dst.proof_id);
        if $src.state == TASK_STATE_FAILED
            || $src.state == TASK_STATE_SUCCESS
            || $src.state == TASK_STATE_UNPROCESSED
        {
            $dst.state = $src.state;
            if TASK_STATE_UNPROCESSED != $src.state {
                $dst.trace.finish_ts = get_timestamp();
                $src.trace.finish_ts = $dst.trace.finish_ts;

                // Fill in the output of the source task.
                $dst.output = $src.output.clone();
            }
            if TASK_STATE_FAILED == $src.state {
                $stage.is_error = true;
            }
        }
    };
}

macro_rules! get_task {
    ($src:ident) => {
        if $src.state == TASK_STATE_UNPROCESSED || $src.state == TASK_STATE_FAILED {
            $src.state = TASK_STATE_PROCESSING;
            $src.trace.start_ts = get_timestamp();
            return Some($src.clone());
        }
        return None
    };
}

impl Stage {
    pub fn new(generate_task: GenerateTask) -> Self {
        Stage {
            //base_dir: generate_task.base_dir.clone(),
            generate_task,
            split_task: SplitTask::default(),
            prove_tasks: Vec::new(),
            agg_tasks: Vec::new(),
            snark_task: SnarkTask::default(),
            is_error: false,
            errmsg: "".to_string(),
            step: Step::Init,
        }
    }

    pub fn dispatch(&mut self) {
        match self.step {
            Step::Init => {
                self.gen_split_task();
                self.step = Step::InSplit;
            }
            Step::InSplit => {
                if self.split_task.state == TASK_STATE_SUCCESS {
                    if self.generate_task.execute_only {
                        self.step = Step::End;
                    } else {
                        self.gen_prove_task();
                        self.step = Step::InProve;
                    }
                }
            }
            Step::InProve => {
                if self
                    .prove_tasks
                    .iter()
                    .all(|task| task.state == TASK_STATE_SUCCESS)
                {
                    if self.generate_task.composite_proof {
                        self.step = Step::End;
                    } else {
                        self.gen_agg_tasks();
                        self.step = Step::InAgg;
                    }
                    // TODO: we will deprecate the agg_all prover.
                    //} else if self.prove_tasks.len() > 3 {
                    //    self.gen_agg_tasks();
                    //    self.step = Step::InAgg;
                    //} else {
                    //    self.gen_agg_all_task();
                    //    self.step = Step::InAggAll;
                    //}
                }
            }
            Step::InAgg => {
                if self
                    .agg_tasks
                    .iter()
                    .all(|task| task.state == TASK_STATE_SUCCESS)
                {
                    self.gen_snark_task();
                    self.step = Step::InSnark;
                }
            }
            Step::InSnark => {
                if self.snark_task.state == TASK_STATE_SUCCESS {
                    self.step = Step::End;
                }
            }
            _ => {}
        }
    }

    pub fn is_success(&mut self) -> bool {
        if self.step == Step::End || self.snark_task.state == TASK_STATE_SUCCESS {
            return true;
        }
        false
    }

    pub fn is_error(&self) -> bool {
        self.is_error
    }

    fn gen_split_task(&mut self) {
        assert!(self.split_task.state == TASK_STATE_INITIAL);
        self.split_task
            .proof_id
            .clone_from(&self.generate_task.proof_id);
        self.split_task
            .base_dir
            .clone_from(&self.generate_task.base_dir);
        self.split_task
            .elf_path
            .clone_from(&self.generate_task.elf_path);
        self.split_task
            .seg_path
            .clone_from(&self.generate_task.seg_path);
        self.split_task
            .public_input_path
            .clone_from(&self.generate_task.public_input_path);
        self.split_task
            .private_input_path
            .clone_from(&self.generate_task.private_input_path);
        self.split_task
            .output_path
            .clone_from(&self.generate_task.output_stream_path);
        self.split_task.block_no = self.generate_task.block_no;
        self.split_task.seg_size = self.generate_task.seg_size;

        self.split_task.task_id = uuid::Uuid::new_v4().to_string();
        self.split_task.state = TASK_STATE_UNPROCESSED;
    }

    pub fn get_split_task(&mut self) -> Option<SplitTask> {
        let src = &mut self.split_task;
        get_task!(src);
    }

    pub fn on_split_task(&mut self, split_task: &mut SplitTask) {
        let dst = &mut self.split_task;
        dst.total_steps = split_task.total_steps;
        on_task!(split_task, dst, self);
    }

    fn gen_prove_task(&mut self) {
        //let prove_dir = self.generate_task.prove_path(true);
        log::info!("ProveTask: {:?}", self.generate_task);
        let files = file::new(&self.generate_task.seg_path).read_dir().unwrap();
        // Read the segment and put them in queue.
        for file_name in files {
            let result = file_name.parse::<usize>();
            if let Ok(file_no) = result {
                let prove_task = ProveTask {
                    task_id: uuid::Uuid::new_v4().to_string(), // FIXME: Do you need it?
                    state: TASK_STATE_UNPROCESSED,
                    trace: Trace::default(),
                    base_dir: self.generate_task.base_dir.clone(),
                    file_no,
                    segment: safe_read(&format!("{}/{file_name}", self.generate_task.seg_path)),
                    program: self.generate_task.gen_program(),
                    // will be assigned after the root proving
                    output: vec![],
                };
                self.prove_tasks.push(prove_task);
            }
        }
        self.prove_tasks.sort_by_key(|p| p.file_no);

        if self.prove_tasks.len() < 2 {
            self.is_error = true;
            self.errmsg = format!(
                "Segment count is {}, please reduce SEG_SIZE !",
                self.prove_tasks.len()
            );
        }
    }

    pub fn get_prove_task(&mut self) -> Option<ProveTask> {
        for prove_task in &mut self.prove_tasks {
            if prove_task.state == TASK_STATE_UNPROCESSED || prove_task.state == TASK_STATE_FAILED {
                prove_task.state = TASK_STATE_PROCESSING;
                prove_task.trace.start_ts = get_timestamp();
                return Some(prove_task.clone());
            }
        }
        None
    }

    pub fn on_prove_task(&mut self, prove_task: &mut ProveTask) {
        for mut item_task in &mut self.prove_tasks {
            if item_task.task_id == prove_task.task_id && item_task.state == TASK_STATE_PROCESSING {
                let dst = &mut item_task;
                on_task!(prove_task, dst, self);
                break;
            }
        }
    }

    pub fn gen_agg_tasks(&mut self) {
        // FIXME: we don't have to wait all the prove tasks done for the single GenerateTask. We should keep track of the agg_index in the Stage structure.
        let mut agg_index = 0;
        let mut result = Vec::new();
        let mut current_length = self.prove_tasks.len();
        for i in (0..current_length - 1).step_by(2) {
            agg_index += 1;
            result.push(agg_task::AggTask::init_from_two_prove_task(
                &(self.prove_tasks[i]),
                &(self.prove_tasks[i + 1]),
                agg_index,
            ));
        }
        if current_length % 2 == 1 {
            result.push(agg_task::AggTask::init_from_single_prove_task(
                &(self.prove_tasks[current_length - 1]),
                agg_index,
            ));
        }
        self.agg_tasks.append(&mut result.clone());

        current_length = result.len();
        while current_length > 1 {
            let mut new_result = Vec::new();
            for i in (0..current_length - 1).step_by(2) {
                agg_index += 1;
                let agg_task = agg_task::AggTask::init_from_two_agg_task(
                    &result[i],
                    &result[i + 1],
                    agg_index,
                );
                self.agg_tasks.push(agg_task.clone());
                new_result.push(agg_task);
            }
            if current_length % 2 == 1 {
                new_result.push(result[current_length - 1].clone());
            }
            result = new_result;
            current_length = result.len();
        }
        let last_agg_tasks = self.agg_tasks.len() - 1;
        self.agg_tasks[last_agg_tasks].is_final = true;
    }

    pub fn get_agg_task(&mut self) -> Option<AggTask> {
        let mut result: Option<AggTask> = None;
        log::info!("get_aag_task: {:?}", self.agg_tasks.len());
        for agg_task in &mut self.agg_tasks {
            if agg_task.left.is_some() || agg_task.right.is_some() {
                log::info!(
                    "get_aag_task: left: {:?}, right: {:?}",
                    agg_task.left.is_some(),
                    agg_task.right.is_some()
                );
                continue;
            }
            if agg_task.state == TASK_STATE_UNPROCESSED || agg_task.state == TASK_STATE_FAILED {
                agg_task.state = TASK_STATE_PROCESSING;
                agg_task.trace.start_ts = get_timestamp();
                result = Some(agg_task.clone());
                break;
            }
        }
        // Fill in the input1/2
        match &mut result {
            Some(agg_task) => {
                let input1 = &mut agg_task.input1;
                let input2 = &mut agg_task.input2;
                vec![input1, input2].iter_mut().for_each(|input| {
                    if input.is_agg {
                        let tmp = self
                            .agg_tasks
                            .iter()
                            .find(|x| x.task_id == input.computed_request_id)
                            .unwrap();
                        input.receipt_input = tmp.output.clone();
                    } else {
                        let tmp = self
                            .prove_tasks
                            .iter()
                            .find(|x| x.task_id == input.computed_request_id)
                            .unwrap();
                        input.receipt_input = tmp.output.clone();
                    }
                });
                log::info!(
                    "to_agg_task: {:?}, {:?}",
                    agg_task.input1.receipt_input.len(),
                    agg_task.input2.receipt_input.len()
                );
            }
            _ => {}
        };
        log::info!("get_aag_task:yes? {:?}", result.is_some());
        result
    }

    pub fn on_agg_task(&mut self, agg_task: &mut AggTask) {
        for item_task in &mut self.agg_tasks {
            if item_task.task_id == agg_task.task_id && item_task.state == TASK_STATE_PROCESSING {
                on_task!(agg_task, item_task, self);
                break;
            }
        }
        if agg_task.state == TASK_STATE_SUCCESS {
            for item_task in &mut self.agg_tasks {
                if item_task.clear_child_task(&agg_task.task_id) {
                    break;
                }
            }
        }
    }

    pub fn gen_snark_task(&mut self) {
        assert!(self.snark_task.state == TASK_STATE_INITIAL);
        self.snark_task
            .proof_id
            .clone_from(&self.generate_task.proof_id.clone());
        self.snark_task
            .input_dir
            .clone_from(&self.generate_task.agg_path);
        self.snark_task
            .output_path
            .clone_from(&self.generate_task.snark_path);
        self.snark_task.task_id = uuid::Uuid::new_v4().to_string();
        self.snark_task.state = TASK_STATE_UNPROCESSED;
        // fill in the input receipts
        for agg_task in &self.agg_tasks {
            if agg_task.is_final == true {
                self.snark_task.agg_receipt = agg_task.output.clone();
            }
        }
        log::info!(
            "gen_snark_task: {:?} {:?}",
            self.snark_task.proof_id,
            self.snark_task.task_id
        );
    }

    pub fn get_snark_task(&mut self) -> Option<SnarkTask> {
        let src = &mut self.snark_task;
        log::info!("get_snark_task: {:?} {:?}", src.proof_id, src.task_id);
        get_task!(src);
    }

    pub fn on_snark_task(&mut self, snark_task: &mut SnarkTask) {
        let dst = &mut self.snark_task;
        on_task!(snark_task, dst, self);
    }
}

impl Debug for Stage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let split_cost = format!(
            "split_id: {} cost: {} sec",
            self.split_task.task_id,
            self.split_task.trace.duration(),
        );
        let root_prove_cost = self
            .prove_tasks
            .iter()
            .map(|task| {
                format!(
                    "prove_id: {} cost: {} sec",
                    task.task_id,
                    task.trace.duration(),
                )
            })
            .collect::<Vec<String>>()
            .join(" \r\n");
        let agg_cost = self
            .agg_tasks
            .iter()
            .map(|task| {
                format!(
                    "agg_id: {} cost: {} sec",
                    task.task_id,
                    task.trace.duration(),
                )
            })
            .collect::<Vec<String>>()
            .join(" \r\n");
        let snark_cost = format!(
            "snark_id: {} cost: {} sec",
            self.snark_task.task_id,
            self.snark_task.trace.duration(),
        );

        write!(
            f,
            "proof_id: {}\r\n {}\r\n {}\r\n {}\r\n {}\r\n",
            self.generate_task.proof_id, split_cost, root_prove_cost, agg_cost, snark_cost
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_gen_agg_tasks() {
        for n in 12..20 {
            let mut stage = Stage {
                ..Default::default()
            };
            for i in 0..n {
                stage.prove_tasks.push(ProveTask {
                    output: vec![1, 2, 3],
                    file_no: i,
                    ..Default::default()
                })
            }
            stage.gen_agg_tasks();
            stage.agg_tasks.iter().for_each(|element| {
                println!(
                    "agg: left:{} right:{} final:{}",
                    //element.file_key,
                    element.input1.is_agg,
                    element.input2.is_agg,
                    element.is_final,
                );
            });
            assert!(stage.agg_tasks.len() <= n);
        }
    }
}
