use std::fs::File;  
use std::io::Write;
use std::io::Read;

pub fn read_file_content(path: &String) -> anyhow::Result<String> {
    let mut file_root = File::open(path)?;
    let mut content = String::new();  
    file_root.read_to_string(&mut content)?;
    return Ok(content)  
}