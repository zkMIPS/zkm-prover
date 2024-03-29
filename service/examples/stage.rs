use stage_service::stage_service_client::StageServiceClient;
use stage_service::{BlockFileItem, GenerateProofRequest};

use std::env;
use std::fs;
use std::path::Path;

use std::time::Instant;

pub mod stage_service {
    tonic::include_proto!("stage.v1");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let elf_path = env::var("ELF_PATH").unwrap_or("/tmp/zkm/test/hello_world".to_string());
    let block_path = env::var("BLOCK_PATH").unwrap_or("/tmp/zkm/test/0_13284491".to_string());
    let block_no = env::var("BLOCK_NO").unwrap_or("13284491".to_string());
    let block_no = block_no.parse::<_>().unwrap_or(13284491);
    let seg_size = env::var("SEG_SIZE").unwrap_or("262144".to_string());
    let seg_size = seg_size.parse::<_>().unwrap_or(262144);

    let elf_data = prover::provers::read_file_bin(&elf_path).unwrap();
    let mut block_data = Vec::new();

    let block_dir_path = Path::new(&block_path);
    let dir_entries = fs::read_dir(block_dir_path).unwrap();
    for entry in dir_entries {
        let entry = entry.unwrap();
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let file_path = format!("{}/{}", block_path, file_name);
        let block_file_item = BlockFileItem {
            file_name: file_name.to_string(),
            file_content: prover::provers::read_file_bin(&file_path).unwrap(),
        };
        block_data.push(block_file_item);
    }

    let request = GenerateProofRequest {
        proof_id: uuid::Uuid::new_v4().to_string(),
        elf_data,
        block_data,
        block_no,
        seg_size,
        ..Default::default()
    };
    println!("request: {:?}", request.proof_id.clone());
    let start = Instant::now();
    let mut stage_client = StageServiceClient::connect("http://127.0.0.1:50000").await?;
    let response = stage_client.generate_proof(request).await?.into_inner();
    println!("response: {:?}", response);
    let end = Instant::now();
    let elapsed = end.duration_since(start);
    println!("Elapsed time: {:?} secs", elapsed.as_secs());
    Ok(())
}
