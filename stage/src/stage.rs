use std::fs;  
use std::io;  
use std::path::Path;
use std::result;
use std::str::FromStr;
use std::fs::File; 
use std::io::Write;
use std::io::Read;
use std::sync::Mutex;  
use std::sync::Arc;
use crate::contexts::generate_context;
use crate::tasks::TASK_STATE_PROCESSING;
use crate::{contexts::generate_context::GenerateContext};
use crate::tasks::{TASK_STATE_FAILED, TASK_STATE_INITIAL, TASK_STATE_SUCCESS, TASK_STATE_UNPROCESSED};
use crate::tasks::{split_task, final_task, SplitTask, FinalTask, prove_task, agg_task, AggAllTask, ProveTask};

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
            if self.prove_tasks.len() == 0 {
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
        if self.agg_all_task.state == TASK_STATE_SUCCESS {
            if self.final_task.state == TASK_STATE_INITIAL {
                self.gen_final_task();
                return;
            }
        } else {
            return;
        }
    }

    pub fn is_success(&mut self) -> bool {
        // TODO
        if self.agg_all_task.state == TASK_STATE_SUCCESS {
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
        println!("gen_split_task {:#?}", self.split_task);
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
        if split_task.state == TASK_STATE_FAILED || split_task.state == TASK_STATE_SUCCESS || split_task.state == TASK_STATE_UNPROCESSED{
            self.split_task.state = split_task.state;
            if TASK_STATE_UNPROCESSED != split_task.state {
                println!("on_split_task {:#?}", self.split_task);
            }
        }
    }

    fn gen_prove_task (&mut self) {
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
                let mut prove_task = ProveTask::default();
                prove_task.task_id = uuid::Uuid::new_v4().to_string();
                prove_task.base_dir = self.generate_context.basedir.clone();
                prove_task.block_no = self.generate_context.block_no;
                prove_task.seg_size = self.generate_context.seg_size;
                prove_task.proof_id = self.generate_context.proof_id.clone();
                prove_task.prove_path = format!("{}/proof/{}", prove_dir.clone(), file_no);
                prove_task.pub_value_path = format!("{}/pub_value/{}", prove_dir.clone(), file_no);
                prove_task.seg_path = format!("{}/{}",self.generate_context.seg_path, file_name.to_string());
                prove_task.state = TASK_STATE_UNPROCESSED;
                self.prove_tasks.push(prove_task);
            }
        }
        println!("gen_prove_task {:#?}", self.prove_tasks);
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
        if prove_task.state == TASK_STATE_FAILED || prove_task.state == TASK_STATE_SUCCESS || prove_task.state == TASK_STATE_UNPROCESSED {
            if TASK_STATE_UNPROCESSED != prove_task.state {
                println!("on_prove_task {:#?}", prove_task);
            }
            for item_task in &mut self.prove_tasks {
                if item_task.task_id == prove_task.task_id && item_task.state == TASK_STATE_PROCESSING {
                    item_task.state = prove_task.state;
                    break;
                }
            }
        }
    }

    pub fn gen_agg_all_task (&mut self) {
        assert!(self.agg_all_task.state == TASK_STATE_INITIAL);
        self.agg_all_task.task_id = uuid::Uuid::new_v4().to_string();
        self.agg_all_task.state = TASK_STATE_UNPROCESSED;
        self.agg_all_task.base_dir = self.generate_context.basedir.clone();
        self.agg_all_task.block_no = self.generate_context.block_no;
        self.agg_all_task.seg_size = self.generate_context.seg_size;
        self.agg_all_task.proof_id = self.generate_context.proof_id.clone(); 
        self.agg_all_task.proof_num = self.prove_tasks.len() as u32;
        self.agg_all_task.proof_dir = format!("{}/proof", self.generate_context.prove_path);
        self.agg_all_task.pub_value_dir = format!("{}/pub_value", self.generate_context.prove_path);
        self.agg_all_task.output_dir = format!("{}", self.generate_context.agg_path);
        println!("gen_agg_task {:#?}", self.agg_all_task);
    }

    pub fn get_agg_all_task(&mut self) -> Option<AggAllTask> {
        if self.agg_all_task.state == TASK_STATE_UNPROCESSED || 
            self.agg_all_task.state == TASK_STATE_FAILED {
            self.agg_all_task.state = TASK_STATE_PROCESSING;
            return Some(self.agg_all_task.clone()); 
        }
        return None
    }

    pub fn on_agg_all_task(&mut self, agg_all_task: AggAllTask) {
        assert!(agg_all_task.proof_id == self.agg_all_task.proof_id);
        if agg_all_task.state == TASK_STATE_FAILED || agg_all_task.state == TASK_STATE_SUCCESS || agg_all_task.state == TASK_STATE_UNPROCESSED{
            self.agg_all_task.state = agg_all_task.state;
            if TASK_STATE_UNPROCESSED != agg_all_task.state {
                println!("on_agg_all_task {:#?}", self.agg_all_task);
            }
        }
    }

    pub fn gen_final_task(&mut self) {
        
    }

    pub fn get_final_task(&mut self) -> Option<FinalTask> {
        return None;
    }

    pub fn on_final_task(&mut self, final_task: FinalTask) {
        
    }
}