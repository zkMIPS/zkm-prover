// pub mod distributed_concurrency;
pub mod concurrency;

pub fn get_block_path(basedir: &str, block: &str, file: &str) -> String {
    format!("{basedir}/0_{block}/{file}")
}
