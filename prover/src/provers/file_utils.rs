use std::fs::File;
use std::io::Read;

pub fn read_file_content(path: &String) -> anyhow::Result<String> {
    let mut file_root = File::open(path)?;
    let mut content = String::new();
    file_root.read_to_string(&mut content)?;
    Ok(content)
}

pub fn read_file_bin(path: &String) -> anyhow::Result<Vec<u8>> {
    let mut file_root = File::open(path)?;
    let mut content = Vec::new();
    file_root.read_to_end(&mut content)?;
    Ok(content)
}
