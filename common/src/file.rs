use aws_sdk_s3::primitives::ByteStream;
use tokio::io::AsyncReadExt;

use futures::executor::block_on;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;

use anyhow::Ok;

pub fn read(path: &str) -> anyhow::Result<Vec<u8>> {
    if is_s3_path(path) {
        return block_on(async { s3_read(path).await });
    }
    Ok(std::fs::read(path)?)
}

async fn s3_read(path: &str) -> anyhow::Result<Vec<u8>> {
    let (bucket, key) = parse_s3_path(path);
    let client = get_s3_client().await;

    let response = client.get_object().bucket(bucket).key(key).send().await?;

    let mut vec_bytes = Vec::new();
    response
        .body
        .into_async_read()
        .read_to_end(&mut vec_bytes)
        .await?;

    Ok(vec_bytes)
}

pub fn read_to_string(path: &str) -> anyhow::Result<String> {
    if is_s3_path(path) {
        let data = read(path)?;
        return Ok(String::from_utf8(data)?);
    }
    let mut file_root = File::open(path)?;
    let mut content = String::new();
    file_root.read_to_string(&mut content)?;
    Ok(content)
}

pub fn create_dir_all(path: &str) -> anyhow::Result<()> {
    if is_s3_path(path) {
        return block_on(async { s3_create_dir_all(path).await });
    }
    fs::create_dir_all(path)?;
    Ok(())
}

async fn s3_create_dir_all(path: &str) -> anyhow::Result<()> {
    let (bucket, key) = parse_s3_path(path);
    let parts: Vec<&str> = key.split('/').collect();
    let mut path = format!("s3://{}", bucket);
    for part in parts {
        if part.is_empty() {
            continue;
        }
        path.push('/');
        path.push_str(part);
        let exist = s3_exist(&path).await?;
        if !exist {
            s3_write_file(&path, &[]).await?;
        }
    }
    Ok(())
}

pub fn write_file(path: &str, buf: &[u8]) -> anyhow::Result<()> {
    if is_s3_path(path) {
        return block_on(async { s3_write_file(path, buf).await });
    }
    let mut file = File::create(path)?;
    file.write_all(buf)?;
    file.flush()?;

    Ok(())
}

async fn s3_write_file(path: &str, buf: &[u8]) -> anyhow::Result<()> {
    let (bucket, key) = parse_s3_path(path);

    let client = get_s3_client().await;
    client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(ByteStream::from(buf.to_vec()))
        .send()
        .await?;

    Ok(())
}

// list_files will return files of current dir
pub fn list_files(path: &str) -> anyhow::Result<Vec<String>> {
    if is_s3_path(path) {
        return block_on(async { list_files_in_s3(path).await });
    }
    let mut files = vec![];
    let dir_entries = fs::read_dir(path)?;
    for entry in dir_entries {
        let entry = entry?;
        let path = entry.path();
        if let Some(file_name) = path.file_name() {
            if let Some(file_name) = file_name.to_str() {
                files.push(file_name.to_string());
            }
        }
    }
    Ok(files)
}

async fn list_files_in_s3(path: &str) -> anyhow::Result<Vec<String>> {
    let (bucket, key) = parse_s3_path(path);
    let client = get_s3_client().await;
    let prefix = if key.ends_with('/') {
        key
    } else {
        format!("{}/", key)
    };
    let response = client
        .list_objects_v2()
        .bucket(bucket)
        .prefix(prefix)
        .delimiter("/".to_string())
        .send()
        .await?;
    let mut files = vec![];
    if let Some(contents) = response.contents {
        for object in contents {
            if let Some(key) = object.key {
                if let Some((_, file)) = key.rsplit_once('/') {
                    files.push(file.to_string());
                } else {
                    files.push(key);
                }
            }
        }
    }
    Ok(files)
}

pub async fn s3_exist(path: &str) -> anyhow::Result<bool> {
    let (bucket, key) = parse_s3_path(path);
    let client = get_s3_client().await;

    let response = client.head_object().bucket(bucket).key(key).send().await;

    Ok(response.is_ok())
}

/// parse_s3_path read a s3 path and return bucket and object key
pub fn parse_s3_path(path: &str) -> (String, String) {
    let path_without_prefix = path.strip_prefix("s3://").unwrap();
    let (bucket, key) = path_without_prefix.split_once('/').unwrap();
    (bucket.to_string(), key.to_string())
}

pub fn is_s3_path(path: &str) -> bool {
    path.starts_with("s3://")
}

async fn get_s3_client() -> aws_sdk_s3::Client {
    // todo: find a way to load from config file
    let config = aws_config::load_from_env().await;
    aws_sdk_s3::Client::new(&config)
}
