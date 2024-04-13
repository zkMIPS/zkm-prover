use anyhow::Ok;
use aws_sdk_s3::primitives::ByteStream;
use std::fs;
use std::io::Read;
use std::io::Write;
use std::thread;
use tokio::io::AsyncReadExt;
use tokio::runtime::Runtime;

pub fn new(path: &str) -> Box<dyn File> {
    if is_s3_path(path) {
        return Box::new(S3File::new(path));
    }
    Box::new(LocalFile::new(path))
}

pub fn is_s3_path(path: &str) -> bool {
    path.starts_with("s3://")
}

pub trait File: std::io::Write {
    fn read(&self) -> anyhow::Result<Vec<u8>>;
    fn read_to_string(&self) -> anyhow::Result<String>;
    fn read_dir(&self) -> anyhow::Result<Vec<String>>;
    fn create_dir_all(&self) -> anyhow::Result<()>;
}

pub struct LocalFile {
    pub path: String,
}

impl LocalFile {
    pub fn new(path: &str) -> Self {
        LocalFile {
            path: path.to_string(),
        }
    }
}

impl Write for LocalFile {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut file = std::fs::File::create(&self.path)?;
        file.write_all(buf)?;
        file.flush()?;

        std::result::Result::Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        std::result::Result::Ok(())
    }
}

impl File for LocalFile {
    fn read(&self) -> anyhow::Result<Vec<u8>> {
        Ok(std::fs::read(&self.path)?)
    }

    fn read_to_string(&self) -> anyhow::Result<String> {
        let mut file_root = std::fs::File::open(&self.path)?;
        let mut content = String::new();
        file_root.read_to_string(&mut content)?;
        Ok(content)
    }

    fn read_dir(&self) -> anyhow::Result<Vec<String>> {
        let mut files = vec![];
        let dir_entries = fs::read_dir(&self.path)?;
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

    fn create_dir_all(&self) -> anyhow::Result<()> {
        fs::create_dir_all(&self.path)?;
        Ok(())
    }
}

pub struct S3File {
    pub path: String,
}

impl S3File {
    pub fn new(path: &str) -> Self {
        S3File {
            path: path.to_string(),
        }
    }
}

impl Write for S3File {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let path = self.path.clone();
        let length = buf.len();
        let buf = buf.to_vec();
        let handle = thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async { s3_write_file(&path, &buf).await })
        });

        handle
            .join()
            .unwrap()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("{}", e)))?;
        std::result::Result::Ok(length)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        std::result::Result::Ok(())
    }
}

impl File for S3File {
    fn read(&self) -> anyhow::Result<Vec<u8>> {
        let path = self.path.clone();
        let handle = thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async { s3_read(&path).await })
        });

        handle.join().unwrap()
    }

    fn read_to_string(&self) -> anyhow::Result<String> {
        let data = self.read()?;
        Ok(String::from_utf8(data)?)
    }

    fn read_dir(&self) -> anyhow::Result<Vec<String>> {
        let path = self.path.clone();
        let handle = thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async { list_files_in_s3(&path).await })
        });

        handle.join().unwrap()
    }

    fn create_dir_all(&self) -> anyhow::Result<()> {
        let path = self.path.clone();
        let handle = thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async { s3_create_dir_all(&path).await })
        });

        handle.join().unwrap()
    }
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

async fn s3_exist(path: &str) -> anyhow::Result<bool> {
    let (bucket, key) = parse_s3_path(path);
    let client = get_s3_client().await;

    let response = client.head_object().bucket(bucket).key(key).send().await;

    Ok(response.is_ok())
}

// parse_s3_path read a s3 path and return bucket and object key
fn parse_s3_path(path: &str) -> (String, String) {
    let path_without_prefix = path.strip_prefix("s3://").unwrap();
    let (bucket, key) = path_without_prefix.split_once('/').unwrap();
    (bucket.to_string(), key.to_string())
}

async fn get_s3_client() -> aws_sdk_s3::Client {
    let config = aws_config::load_from_env().await;
    aws_sdk_s3::Client::new(&config)
}
