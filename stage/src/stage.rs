use std::fs;  
use std::io;  
use std::path::Path;
use std::fs::File; 
use std::io::Write;
use std::sync::Mutex;  
use std::sync::Arc;
use crate::contexts::generate_context;
use crate::tasks::TASK_STATE_PROCESSING;
use crate::{contexts::generate_context::GenerateContext};
use crate::tasks::{TASK_STATE_FAILED, TASK_STATE_INITIAL, TASK_STATE_SUCCESS, TASK_STATE_UNPROCESSED};
use crate::tasks::{split_task, final_task, SplitTask, FinalTask, prove_task, agg_task, AggTask, ProveTask};

pub struct Stage {
    pub generate_context: GenerateContext,
    pub split_task: SplitTask,
    pub prove_tasks: Vec<ProveTask>,
    pub agg_tasks: Vec<AggTask>,
    pub final_task: FinalTask,
}

impl Stage {
    pub fn new(generate_context: GenerateContext) -> Self {
        Stage {
            generate_context,
            split_task: SplitTask::default(),
            prove_tasks: Vec::new(),
            agg_tasks: Vec::new(),
            final_task: FinalTask::default(),
        }
    }

    pub fn dispatch(&mut self) {
        if self.split_task.state == TASK_STATE_INITIAL {
            self.gen_split_task();
            return;
        }
        if self.split_task.state == TASK_STATE_SUCCESS {
            if self.prove_tasks.len() == 0 {
                self.gen_prove_task();
                return;
            }
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
        if all_prove_task_success && self.agg_tasks.len() == 0 {
            self.gen_agg_task();
            return;
        }
        let mut all_agg_task_success = true;
        for agg_task in &self.agg_tasks {
            if agg_task.state != TASK_STATE_SUCCESS {
                all_agg_task_success = false;
                break;
            }
        }
        if !all_agg_task_success {
            return;
        }
        if all_agg_task_success && self.final_task.state == TASK_STATE_INITIAL {
            self.gen_final_task();
            return;
        }
    }

    pub fn is_success(&mut self) -> bool {
        if self.final_task.state == TASK_STATE_SUCCESS {
            return true;
        }
        return false;
    }

    fn gen_split_task (&mut self) {
        assert!(self.split_task.state == TASK_STATE_INITIAL);
        self.split_task.proof_id = self.generate_context.proof_id.clone();
        self.split_task.base_dir = self.generate_context.basedir.clone();
        self.split_task.elf_path = self.generate_context.elf_path.clone();
        self.split_task.seg_path = self.generate_context.seg_path.clone();
        self.split_task.block_no = self.generate_context.block_no;
        self.split_task.seg_size = self.generate_context.seg_size;
        self.split_task.task_id = uuid::Uuid::new_v4().to_string();
        self.split_task.state = TASK_STATE_UNPROCESSED;
        print!("gen_split_task {:?}", self.split_task);
    }

    pub fn get_split_task(&mut self) -> Option<SplitTask> {
        if self.split_task.state == TASK_STATE_UNPROCESSED || 
            self.split_task.state == TASK_STATE_FAILED {
            self.split_task.state = TASK_STATE_PROCESSING;
            return Some(self.split_task.clone()); 
        }
        return None
    }

    pub fn on_split_task(&mut self, split_task: SplitTask) {
        assert!(split_task.proof_id == self.split_task.proof_id);
        if split_task.state == TASK_STATE_FAILED || split_task.state == TASK_STATE_SUCCESS {
            self.split_task.state = split_task.state;
            print!("on_split_task {:?}", self.split_task);
        }
    }

    fn gen_prove_task (&mut self) {
        let seg_dir_path = Path::new(&self.generate_context.seg_path);
        let dir_entries = fs::read_dir(seg_dir_path).unwrap();    
        for entry in dir_entries {  
            let entry = entry.unwrap();  
            let path = entry.path();
            let file_name = path.file_name().unwrap().to_str().unwrap(); 
            let mut prove_task = ProveTask::default();
            prove_task.task_id = uuid::Uuid::new_v4().to_string();
            prove_task.base_dir = self.generate_context.basedir.clone();
            prove_task.block_no = self.generate_context.block_no;
            prove_task.seg_size = self.generate_context.seg_size;
            prove_task.proof_id = self.generate_context.proof_id.clone();
            prove_task.prove_path = format!("{}/prove_{}", self.generate_context.prove_path.clone(), prove_task.task_id);
            prove_task.pub_value_path = format!("{}/pub_value_{}", self.generate_context.prove_path.clone(), prove_task.task_id);
            prove_task.seg_path = format!("{}/{}",self.generate_context.seg_path, file_name.to_string());
            prove_task.state = TASK_STATE_UNPROCESSED;
            self.prove_tasks.push(prove_task);
        }
        print!("gen_prove_task {:?}", self.prove_tasks);
    }

    pub fn get_prove_task(&mut self) -> Option<ProveTask> {
        for prove_task in &mut self.prove_tasks {
            if prove_task.state == TASK_STATE_UNPROCESSED || 
                prove_task.state == TASK_STATE_FAILED {
                prove_task.state = TASK_STATE_PROCESSING;
                return Some(prove_task.clone());
            }
        }
        return None
    }

    pub fn on_prove_task(&mut self, prove_task: ProveTask) {
        assert!(prove_task.proof_id == self.generate_context.proof_id);
        if prove_task.state == TASK_STATE_FAILED || prove_task.state == TASK_STATE_SUCCESS {
            print!("on_split_task {:?}", prove_task);
            for item_task in &mut self.prove_tasks {
                if item_task.task_id == prove_task.task_id && item_task.state == TASK_STATE_PROCESSING {
                    item_task.state = prove_task.state;
                    break;
                }
            }
        }
    }

    pub fn gen_agg_task (&mut self) {
        let dir_path = Path::new(&self.generate_context.prove_path);
        let dir_entries = fs::read_dir(dir_path).unwrap();    
        for entry in dir_entries {
            let entry = entry.unwrap();  
            let path = entry.path();
            let file_name = path.file_name().unwrap().to_str().unwrap(); 
            let mut agg_task = AggTask::default();
            agg_task.task_id = uuid::Uuid::new_v4().to_string();
            agg_task.proof_id = self.generate_context.proof_id.clone(); 
            agg_task.elf_path = self.generate_context.prove_path.clone();
            agg_task.seg_path = file_name.to_string();
            agg_task.state = TASK_STATE_UNPROCESSED;
            self.agg_tasks.push(agg_task);
        }
        print!("gen_agg_task {:?}", self.agg_tasks);
    }

    pub fn get_agg_task(&mut self) -> Option<AggTask> {
        return None
    }

    pub fn on_agg_task(&mut self, agg_task: AggTask) {

    }

    pub fn gen_final_task(&mut self) {

    }

    pub fn get_final_task(&mut self) -> Option<FinalTask> {
        return None
    }

    pub fn on_final_task(&mut self, final_task: FinalTask) {
        
    }
}