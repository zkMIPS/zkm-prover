pub mod stage;
pub mod stage_service;
mod stage_worker;
pub mod tasks;
pub use tasks::generate_task::GenerateTask;

use crate::proto::includes::v1::BlockFileItem;
use common::file;
pub fn read_block_data(block_no: u64, block_path: &str) -> Vec<BlockFileItem> {
    let mut block_data = Vec::new();
    if block_no > 0 {
        let files = file::new(block_path).read_dir().unwrap();
        for file_name in files {
            let file_path = format!("{}/{}", block_path, file_name);
            let block_file_item = BlockFileItem {
                file_name: file_name.to_string(),
                file_content: file::new(&file_path).read().unwrap(),
            };
            block_data.push(block_file_item);
        }
    }
    block_data
}

pub fn safe_read(path: &str) -> Vec<u8> {
    log::info!("read {}", path);
    std::fs::read(path).unwrap_or_else(|_| {
        log::error!("read: {path}, {:?}", backtrace::Backtrace::new());
        panic!("Read {} failed", path);
    })
}
