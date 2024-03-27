use crate::contexts::generate_context::GenerateContext;
use crate::tasks::TASK_STATE_PROCESSING;
use crate::tasks::{AggAllTask, FinalTask, ProveTask, SplitTask};
use crate::tasks::{
    TASK_STATE_FAILED, TASK_STATE_INITIAL, TASK_STATE_SUCCESS, TASK_STATE_UNPROCESSED,
};
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;

pub fn copy_file_bin(src: &String, dst: &String) {
    let mut file_src = File::open(src).unwrap();
    let mut content = Vec::new();
    file_src.read_to_end(&mut content).unwrap();

    let mut file_dst = File::open(dst).unwrap();
    file_dst.write_all(content.as_slice()).unwrap();
}

pub struct Stage {
    pub generate_context: GenerateContext,
    pub split_task: SplitTask,
    pub prove_tasks: Vec<ProveTask>,
    pub agg_all_task: AggAllTask,
    pub final_task: FinalTask,
}

macro_rules! on_task {
    ($src:ident, $dst:ident) => {
        assert!($src.proof_id == $dst.proof_id);
        if $src.state == TASK_STATE_FAILED
            || $src.state == TASK_STATE_SUCCESS
            || $src.state == TASK_STATE_UNPROCESSED
        {
            $dst.state = $src.state;
            if TASK_STATE_UNPROCESSED != $src.state {
                log::info!("on_task {:#?}", $dst);
            }
        }
    };
}

macro_rules! get_task {
    ($src:ident) => {
        if $src.state == TASK_STATE_UNPROCESSED || $src.state == TASK_STATE_FAILED {
            $src.state = TASK_STATE_PROCESSING;
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
            agg_all_task: AggAllTask::default(),
            final_task: FinalTask::default(),
        }
    }

    pub fn dispatch(&mut self) {
        if self.split_task.state == TASK_STATE_INITIAL {
            self.gen_split_task();
            return;
        }
        if self.split_task.state == TASK_STATE_SUCCESS {
            if self.prove_tasks.is_empty() {
                self.gen_prove_task();
                return;
            }
        } else {
            return;
        }
        let mut all_prove_task_success = true;
        for prove_task in &self.prove_tasks {
            if prove_task.state != TASK_STATE_SUCCESS {
                all_prove_task_success = false;
                break;
            }
        }
        if !all_prove_task_success {
            return;
        }
        if all_prove_task_success && self.agg_all_task.state == TASK_STATE_INITIAL {
            self.gen_agg_all_task();
            return;
        }
        if self.agg_all_task.state == TASK_STATE_SUCCESS
            && self.final_task.state == TASK_STATE_INITIAL
        {
            self.gen_final_task();
        }
    }

    pub fn is_success(&mut self) -> bool {
        if self.final_task.state == TASK_STATE_SUCCESS {
            return true;
        }
        false
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

    pub fn on_split_task(&mut self, split_task: SplitTask) {
        let dst = &mut self.split_task;
        on_task!(split_task, dst);
    }

    fn gen_prove_task(&mut self) {
        let prove_dir = self.generate_context.prove_path.clone();
        fs::create_dir_all(prove_dir.clone()).unwrap();
        let seg_dir_path = Path::new(&self.generate_context.seg_path);
        let dir_entries = fs::read_dir(seg_dir_path).unwrap();
        for entry in dir_entries {
            let entry = entry.unwrap();
            let path = entry.path();
            let file_name = path.file_name().unwrap().to_str().unwrap();
            let result: Result<usize, <usize as FromStr>::Err> = file_name.parse();
            if let Ok(file_no) = result {
                let prove_task = ProveTask {
                    task_id: uuid::Uuid::new_v4().to_string(),
                    base_dir: self.generate_context.basedir.clone(),
                    block_no: self.generate_context.block_no,
                    state: TASK_STATE_UNPROCESSED,
                    seg_size: self.generate_context.seg_size,
                    proof_id: self.generate_context.proof_id.clone(),
                    prove_path: format!("{}/proof/{}", prove_dir.clone(), file_no),
                    pub_value_path: format!("{}/pub_value/{}", prove_dir.clone(), file_no),
                    seg_path: format!("{}/{}", self.generate_context.seg_path, file_name),
                };
                self.prove_tasks.push(prove_task);
            }
        }
        log::info!("gen_prove_task {:#?}", self.prove_tasks);
    }

    pub fn get_prove_task(&mut self) -> Option<ProveTask> {
        for prove_task in &mut self.prove_tasks {
            if prove_task.state == TASK_STATE_UNPROCESSED || prove_task.state == TASK_STATE_FAILED {
                prove_task.state = TASK_STATE_PROCESSING;
                return Some(prove_task.clone());
            }
        }
        None
    }

    pub fn on_prove_task(&mut self, prove_task: ProveTask) {
        for mut item_task in &mut self.prove_tasks {
            if item_task.task_id == prove_task.task_id && item_task.state == TASK_STATE_PROCESSING {
                let dst = &mut item_task;
                on_task!(prove_task, dst);
                break;
            }
        }

        assert!(prove_task.proof_id == self.generate_context.proof_id);
        if prove_task.state == TASK_STATE_FAILED
            || prove_task.state == TASK_STATE_SUCCESS
            || prove_task.state == TASK_STATE_UNPROCESSED
        {
            if TASK_STATE_UNPROCESSED != prove_task.state {
                log::info!("on_prove_task {:#?}", prove_task);
            }
            for item_task in &mut self.prove_tasks {
                if item_task.task_id == prove_task.task_id
                    && item_task.state == TASK_STATE_PROCESSING
                {
                    item_task.state = prove_task.state;
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

    pub fn on_agg_all_task(&mut self, agg_all_task: AggAllTask) {
        let dst = &mut self.agg_all_task;
        on_task!(agg_all_task, dst);
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

    pub fn on_final_task(&mut self, final_task: FinalTask) {
        let dst = &mut self.final_task;
        on_task!(final_task, dst);
    }
}
