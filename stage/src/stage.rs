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
use crate::tasks::{split_task, final_task, SplitTask, FinalTask, prove_task, agg_task, AggTask, ProveTask};

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
    pub agg_tasks: Vec<AggTask>,
    pub final_task: FinalTask,
    agg_ids: Vec<usize>,
    agg_level: u32,
}

impl Stage {
    pub fn new(generate_context: GenerateContext) -> Self {
        Stage {
            generate_context,
            split_task: SplitTask::default(),
            prove_tasks: Vec::new(),
            agg_tasks: Vec::new(),
            final_task: FinalTask::default(),
            agg_ids: Vec::new(),
            agg_level: 1,
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
        if all_prove_task_success && self.agg_ids.len() > 1 &&  self.agg_tasks.len() == 0 {
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
        print!("gen_split_task {:#?}", self.split_task);
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
                print!("on_split_task {:#?}", self.split_task);
            }
        }
    }

    fn gen_prove_task (&mut self) {
        let prove_dir = format!("{}/0", self.generate_context.prove_path.clone());
        fs::create_dir_all(prove_dir.clone()).unwrap();
        let seg_dir_path = Path::new(&self.generate_context.seg_path);
        let dir_entries = fs::read_dir(seg_dir_path).unwrap();    
        for entry in dir_entries {  
            let entry = entry.unwrap();  
            let path = entry.path();
            let file_name = path.file_name().unwrap().to_str().unwrap(); 
            let result: Result<usize, <usize as FromStr>::Err> = file_name.parse();
            if let Ok(file_no) = result {
                self.agg_ids.push(file_no);
                let mut prove_task = ProveTask::default();
                prove_task.task_id = uuid::Uuid::new_v4().to_string();
                prove_task.base_dir = self.generate_context.basedir.clone();
                prove_task.block_no = self.generate_context.block_no;
                prove_task.seg_size = self.generate_context.seg_size;
                prove_task.proof_id = self.generate_context.proof_id.clone();
                prove_task.prove_path = format!("{}/prove_{}", prove_dir.clone(), file_no);
                prove_task.pub_value_path = format!("{}/pub_value_{}", prove_dir.clone(), file_no);
                prove_task.seg_path = format!("{}/{}",self.generate_context.seg_path, file_name.to_string());
                prove_task.state = TASK_STATE_UNPROCESSED;
                self.prove_tasks.push(prove_task);
            }
        }
        print!("gen_prove_task {:#?}", self.prove_tasks);
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
                print!("on_prove_task {:#?}", prove_task);
            }
            for item_task in &mut self.prove_tasks {
                if item_task.task_id == prove_task.task_id && item_task.state == TASK_STATE_PROCESSING {
                    item_task.state = prove_task.state;
                    break;
                }
            }
        }
    }

/*
0 1 2 3 4 5
 0   1   2
   0     1
      0

0 1 2 3 4
 0   1  2
   0    1
      0
*/
    pub fn gen_agg_task (&mut self) {
        assert!(self.agg_level > 0);
        self.agg_ids.sort();
        let agg_task_index = 0;
        let agg_level = 0;
        let input_dir = format!("{}/{}", self.generate_context.prove_path.clone(), self.agg_level-1);
        let prove_dir = format!("{}/{}", self.generate_context.prove_path.clone(), self.agg_level);
        fs::create_dir_all(prove_dir.clone()).unwrap();
        let num = self.agg_ids.len();
        loop {
            for i in 0..num / 2  {
                let first = i*2;
                let second = first + 1;
                let mut agg_task = AggTask::default();
                agg_task.task_id = uuid::Uuid::new_v4().to_string();
                agg_task.state = TASK_STATE_UNPROCESSED;
                agg_task.base_dir = self.generate_context.basedir.clone();
                agg_task.block_no = self.generate_context.block_no;
                agg_task.seg_size = self.generate_context.seg_size;
                agg_task.proof_id = self.generate_context.proof_id.clone(); 
                agg_task.seg_path = self.generate_context.seg_path.clone();
                agg_task.proof_path1 = format!("{}/prove_{}", input_dir, first);
                agg_task.pub_value_path1 = format!("{}/pub_value_{}", input_dir, first);
                agg_task.proof_path2 = format!("{}/prove_{}", input_dir, second);
                agg_task.pub_value_path2 = format!("{}/pub_value_{}", input_dir, second);
                agg_task.is_agg_1 = true; // TODO
                agg_task.is_agg_2 = true; // TODO
                agg_task.agg_proof_path = format!("{}/prove_{}", prove_dir, i);
                agg_task.agg_pub_value_path = format!("{}/pub_value_{}", prove_dir, i);
                agg_task.task_seq = i; 
                agg_task.root = i == 0;
                if agg_level > 0 {
                    if let Some(father_1) = self.agg_tasks.get(agg_task_index + first) {
                        agg_task.dependencies.push(father_1.task_id.clone());
                    }
                    if let Some(father_2) = self.agg_tasks.get(agg_task_index + second) {
                        agg_task.dependencies.push(father_2.task_id.clone());
                    }
                }           
                self.agg_tasks.push(agg_task);
            }
            if num % 2 == 1 {
                self.agg_ids.clear();
                let file_no = num / 2 + 1;
                // Copy files to the next loop
                copy_file_bin(&format!("{}/prove_{}", input_dir, num-1), &format!("{}/prove_{}", prove_dir, file_no));
                copy_file_bin(&format!("{}/pub_value_{}", input_dir, num-1), &format!("{}/pub_value_{}", prove_dir, file_no));
                // create a success task
                self.agg_ids.push(file_no);
            }
            if self.agg_ids.len() == 1 {
                break;
            }
        }
        print!("gen_agg_task {:#?}", self.agg_tasks);
    }

    pub fn get_agg_task(&mut self) -> Option<AggTask> {
        for agg_task in &mut self.agg_tasks {
            if agg_task.state == TASK_STATE_UNPROCESSED || 
                agg_task.state == TASK_STATE_FAILED {
                agg_task.state = TASK_STATE_PROCESSING;
                return Some(agg_task.clone());
            }
        }
        return None
    }

    pub fn on_agg_task(&mut self, agg_task: AggTask) {
        assert!(agg_task.proof_id == self.generate_context.proof_id);
        if agg_task.state == TASK_STATE_FAILED || agg_task.state == TASK_STATE_SUCCESS || agg_task.state == TASK_STATE_UNPROCESSED {
            if TASK_STATE_UNPROCESSED != agg_task.state {
                print!("on_agg_task {:#?}", agg_task);
            }
            for item_task in &mut self.agg_tasks {
                if item_task.task_id == agg_task.task_id && item_task.state == TASK_STATE_PROCESSING {
                    item_task.state = agg_task.state;
                    if item_task.state == TASK_STATE_SUCCESS {
                        self.agg_ids.push(item_task.task_seq);
                    }
                    break;
                }
            }
        }
    }

    pub fn gen_final_task(&mut self) {
        assert!(self.agg_ids.len() == 1);
        self.agg_ids.sort();
        let input_dir = format!("{}/{}", self.generate_context.prove_path.clone(), self.agg_level-1);
        let prove_dir = format!("{}/{}", self.generate_context.prove_path.clone(), self.agg_level);
        fs::create_dir_all(prove_dir.clone()).unwrap();
        self.final_task.task_id = uuid::Uuid::new_v4().to_string();
        self.final_task.state = TASK_STATE_UNPROCESSED;
        self.final_task.base_dir = self.generate_context.basedir.clone();
        self.final_task.block_no = self.generate_context.block_no;
        self.final_task.seg_size = self.generate_context.seg_size;
        self.final_task.proof_id = self.generate_context.proof_id.clone(); 
        self.final_task.out_path = self.generate_context.final_path.clone();
        self.final_task.proof_path = format!("{}/prove_{}", input_dir, 0);
        self.final_task.pub_value_path = format!("{}/pub_value_{}", input_dir, 0);
    }

    pub fn get_final_task(&mut self) -> Option<FinalTask> {
        if self.final_task.state == TASK_STATE_UNPROCESSED || 
            self.final_task.state == TASK_STATE_FAILED {
            self.final_task.state = TASK_STATE_PROCESSING;
            return Some(self.final_task.clone()); 
        }
        return None
    }

    pub fn on_final_task(&mut self, final_task: FinalTask) {
        assert!(final_task.proof_id == self.final_task.proof_id);
        if final_task.state == TASK_STATE_FAILED || final_task.state == TASK_STATE_SUCCESS || final_task.state == TASK_STATE_UNPROCESSED{
            self.final_task.state = final_task.state;
            if TASK_STATE_UNPROCESSED != final_task.state {
                print!("on_split_task {:#?}", self.final_task);
            }
        }
    }
}