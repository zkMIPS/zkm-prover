use crate::contexts::generate_context::GenerateContext;
use crate::tasks::agg_task::{self, AggTask};
use crate::tasks::TASK_STATE_PROCESSING;
use crate::tasks::{AggAllTask, FinalTask, ProveTask, SplitTask};
use crate::tasks::{
    TASK_STATE_FAILED, TASK_STATE_INITIAL, TASK_STATE_SUCCESS, TASK_STATE_UNPROCESSED,
};
use common::file;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_timestamp() -> u64 {
    let now = SystemTime::now();
    let duration_since_epoch = now.duration_since(UNIX_EPOCH).unwrap();
    duration_since_epoch.as_secs()
}

#[derive(Default)]
pub enum Step {
    #[default]
    Init,
    InSplit,
    InProve,
    InAgg,
    InAggAll,
    InFinal,
    End,
}

#[derive(Default)]
pub struct Stage {
    pub generate_context: GenerateContext,
    pub split_task: SplitTask,
    pub prove_tasks: Vec<ProveTask>,
    pub agg_tasks: Vec<AggTask>,
    pub agg_all_task: AggAllTask,
    pub final_task: FinalTask,
    pub is_error: bool,
    pub errmsg: String,
    pub step: Step,
}

macro_rules! on_task {
    ($src:ident, $dst:ident, $stage:ident) => {
        assert!($src.proof_id == $dst.proof_id);
        if $src.state == TASK_STATE_FAILED
            || $src.state == TASK_STATE_SUCCESS
            || $src.state == TASK_STATE_UNPROCESSED
        {
            $dst.state = $src.state;
            if TASK_STATE_UNPROCESSED != $src.state {
                log::info!("on_task {:#?}", $dst);
                $dst.finish_ts = get_timestamp();
                $src.finish_ts = $dst.finish_ts;
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
            $src.start_ts = get_timestamp();
            return Some($src.clone());
        }
        return None
    };
}

impl Stage {
    pub fn new(generate_context: GenerateContext) -> Self {
        Stage {
            generate_context,
            split_task: SplitTask::default(),
            prove_tasks: Vec::new(),
            agg_tasks: Vec::new(),
            agg_all_task: AggAllTask::default(),
            final_task: FinalTask::default(),
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
                    self.gen_prove_task();
                    self.step = Step::InProve;
                }
            }
            Step::InProve => {
                if self
                    .prove_tasks
                    .iter()
                    .all(|task| task.state == TASK_STATE_SUCCESS)
                {
                    if self.prove_tasks.len() > 3 {
                        self.gen_agg_tasks();
                        self.step = Step::InAgg;
                    } else {
                        self.gen_agg_all_task();
                        self.step = Step::InAggAll;
                    }
                }
            }
            Step::InAgg => {
                if self
                    .agg_tasks
                    .iter()
                    .all(|task| task.state == TASK_STATE_SUCCESS)
                {
                    self.gen_final_task();
                    self.step = Step::InFinal;
                }
            }
            Step::InAggAll => {
                if self.agg_all_task.state == TASK_STATE_SUCCESS {
                    self.gen_final_task();
                    self.step = Step::InFinal;
                }
            }
            Step::InFinal => {
                if self.final_task.state == TASK_STATE_SUCCESS {
                    self.step = Step::End;
                }
            }
            _ => {}
        }
    }

    pub fn is_success(&mut self) -> bool {
        if self.final_task.state == TASK_STATE_SUCCESS {
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
            .clone_from(&self.generate_context.proof_id);
        self.split_task
            .base_dir
            .clone_from(&self.generate_context.basedir);
        self.split_task
            .elf_path
            .clone_from(&self.generate_context.elf_path);
        self.split_task
            .seg_path
            .clone_from(&self.generate_context.seg_path);
        self.split_task.args.clone_from(&self.generate_context.args);
        self.split_task.block_no = self.generate_context.block_no;
        self.split_task.seg_size = self.generate_context.seg_size;
        self.split_task.task_id = uuid::Uuid::new_v4().to_string();
        self.split_task.state = TASK_STATE_UNPROCESSED;
        log::info!("gen_split_task {:#?}", self.split_task);
    }

    pub fn get_split_task(&mut self) -> Option<SplitTask> {
        let src = &mut self.split_task;
        get_task!(src);
    }

    pub fn on_split_task(&mut self, split_task: &mut SplitTask) {
        let dst = &mut self.split_task;
        on_task!(split_task, dst, self);
    }

    fn gen_prove_task(&mut self) {
        let prove_dir = self.generate_context.prove_path.clone();
        file::new(&prove_dir).create_dir_all().unwrap();
        let files = file::new(&self.generate_context.seg_path)
            .read_dir()
            .unwrap();
        for file_name in files {
            let result: Result<usize, <usize as FromStr>::Err> = file_name.parse();
            if let Ok(file_no) = result {
                let prove_task = ProveTask {
                    file_no,
                    task_id: uuid::Uuid::new_v4().to_string(),
                    base_dir: self.generate_context.basedir.clone(),
                    block_no: self.generate_context.block_no,
                    state: TASK_STATE_UNPROCESSED,
                    seg_size: self.generate_context.seg_size,
                    proof_id: self.generate_context.proof_id.clone(),
                    prove_path: format!("{}/proof/{}", prove_dir.clone(), file_no),
                    pub_value_path: format!("{}/pub_value/{}", prove_dir.clone(), file_no),
                    seg_path: format!("{}/{}", self.generate_context.seg_path, file_name),
                    start_ts: 0,
                    finish_ts: 0,
                    node_info: "".to_string(),
                };
                self.prove_tasks.push(prove_task);
            }
        }
        self.prove_tasks.sort_by_key(|p| p.file_no);
        log::info!("gen_prove_task {:#?}", self.prove_tasks);

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
                prove_task.start_ts = get_timestamp();
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
        let mut agg_index = 0;
        let mut result = Vec::new();
        let mut current_length = self.prove_tasks.len();
        for i in (0..current_length - 1).step_by(2) {
            agg_index += 1;
            result.push(agg_task::AggTask::init_from_two_prove_task(
                &(self.prove_tasks[i]),
                &(self.prove_tasks[i + 1]),
                &self.generate_context.prove_path,
                agg_index,
            ));
        }
        if current_length % 2 == 1 {
            result.push(agg_task::AggTask::init_from_single_prove_task(
                &(self.prove_tasks[current_length - 1]),
                &self.generate_context.prove_path,
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
                    &self.generate_context.prove_path,
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
        self.agg_tasks[last_agg_tasks]
            .output_dir
            .clone_from(&self.generate_context.agg_path);
        log::info!("gen_agg_task {:#?}", self.agg_tasks);
    }

    pub fn get_agg_task(&mut self) -> Option<AggTask> {
        for agg_task in &mut self.agg_tasks {
            if agg_task.left.is_some() || agg_task.right.is_some() {
                continue;
            }
            if agg_task.state == TASK_STATE_UNPROCESSED || agg_task.state == TASK_STATE_FAILED {
                agg_task.state = TASK_STATE_PROCESSING;
                agg_task.start_ts = get_timestamp();
                return Some(agg_task.clone());
            }
        }
        None
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

    pub fn gen_agg_all_task(&mut self) {
        assert!(self.agg_all_task.state == TASK_STATE_INITIAL);
        self.agg_all_task.task_id = uuid::Uuid::new_v4().to_string();
        self.agg_all_task.state = TASK_STATE_UNPROCESSED;
        self.agg_all_task
            .base_dir
            .clone_from(&self.generate_context.basedir);
        self.agg_all_task.block_no = self.generate_context.block_no;
        self.agg_all_task.seg_size = self.generate_context.seg_size;
        self.agg_all_task
            .proof_id
            .clone_from(&self.generate_context.proof_id.clone());
        self.agg_all_task.proof_num = self.prove_tasks.len() as u32;
        self.agg_all_task.proof_dir = format!("{}/proof", self.generate_context.prove_path);
        self.agg_all_task.pub_value_dir = format!("{}/pub_value", self.generate_context.prove_path);
        self.agg_all_task
            .output_dir
            .clone_from(&self.generate_context.agg_path);
        log::info!("gen_agg_task {:#?}", self.agg_all_task);
    }

    pub fn get_agg_all_task(&mut self) -> Option<AggAllTask> {
        let src = &mut self.agg_all_task;
        get_task!(src);
    }

    pub fn on_agg_all_task(&mut self, agg_all_task: &mut AggAllTask) {
        let dst = &mut self.agg_all_task;
        on_task!(agg_all_task, dst, self);
    }

    pub fn gen_final_task(&mut self) {
        assert!(self.final_task.state == TASK_STATE_INITIAL);
        self.final_task
            .proof_id
            .clone_from(&self.generate_context.proof_id.clone());
        self.final_task
            .input_dir
            .clone_from(&self.generate_context.agg_path.clone());
        self.final_task
            .output_path
            .clone_from(&self.generate_context.final_path);
        self.final_task.task_id = uuid::Uuid::new_v4().to_string();
        self.final_task.state = TASK_STATE_UNPROCESSED;
        log::info!("gen_final_task {:#?}", self.final_task);
    }

    pub fn get_final_task(&mut self) -> Option<FinalTask> {
        let src = &mut self.final_task;
        get_task!(src);
    }

    pub fn on_final_task(&mut self, final_task: &mut FinalTask) {
        let dst = &mut self.final_task;
        on_task!(final_task, dst, self);
    }

    pub fn timecost_string(&self) -> String {
        let split_cost = format!(
            "split_id: {} cost: {} sec",
            self.split_task.task_id,
            self.split_task.finish_ts - self.split_task.start_ts
        );
        let root_prove_cost = self
            .prove_tasks
            .iter()
            .map(|task| {
                format!(
                    "prove_id: {} cost: {} sec",
                    task.task_id,
                    task.finish_ts - task.start_ts
                )
            })
            .collect::<Vec<String>>()
            .join("\r\n");
        let agg_all_cost = format!(
            "agg_all_id: {} cost: {} sec",
            self.agg_all_task.task_id,
            self.agg_all_task.finish_ts - self.agg_all_task.start_ts
        );
        let final_cost = format!(
            "final_id: {} cost: {} sec",
            self.final_task.task_id,
            self.final_task.finish_ts - self.final_task.start_ts
        );
        format!(
            "proof_id: {}\r\n{}\r\n{}\r\n{}\r\n{}\r\n",
            self.generate_context.proof_id, split_cost, root_prove_cost, agg_all_cost, final_cost
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
                    file_no: i,
                    ..Default::default()
                })
            }
            stage.gen_agg_tasks();
            stage.agg_tasks.iter().for_each(|element| {
                println!(
                    "key:{} left:{} right:{} final:{}",
                    element.file_key,
                    element.input1.is_agg,
                    element.input2.is_agg,
                    element.is_final,
                );
            });
            assert!(stage.agg_tasks.len() <= n);
        }
    }
}
