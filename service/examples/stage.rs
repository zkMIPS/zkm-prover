use common::file::{read, read_dir};
use common::tls::Config;
use stage_service::stage_service_client::StageServiceClient;
use stage_service::{BlockFileItem, GenerateProofRequest};
use std::env;
use std::time::Instant;
use tonic::transport::ClientTlsConfig;
use tonic::transport::Endpoint;

pub mod stage_service {
    tonic::include_proto!("stage.v1");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let elf_path = env::var("ELF_PATH").unwrap_or("/tmp/zkm/test/hello_world".to_string());
    let block_path = env::var("BLOCK_PATH").unwrap_or("/tmp/zkm/test/0_13284491".to_string());
    let block_no = env::var("BLOCK_NO").unwrap_or("13284491".to_string());
    let block_no = block_no.parse::<_>().unwrap_or(13284491);
    let seg_size = env::var("SEG_SIZE").unwrap_or("16384".to_string());
    let seg_size = seg_size.parse::<_>().unwrap_or(16384);
    let endpoint = env::var("ENDPOINT").unwrap_or("http://127.0.0.1:50000".to_string());
    let ca_cert_path = env::var("CA_CERT_PATH").unwrap_or("".to_string());
    let cert_path = env::var("CERT_PATH").unwrap_or("".to_string());
    let key_path = env::var("KEY_PATH").unwrap_or("".to_string());
    let ssl_config = if ca_cert_path.is_empty() {
        None
    } else {
        Some(Config::new(ca_cert_path, cert_path, key_path).await?)
    };

    let elf_data = read(&elf_path).unwrap();
    let mut block_data = Vec::new();

    let files = read_dir(&block_path).unwrap();
    for file_name in files {
        let file_path = format!("{}/{}", block_path, file_name);
        let block_file_item = BlockFileItem {
            file_name: file_name.to_string(),
            file_content: read(&file_path).unwrap(),
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
    let endpoint = match ssl_config {
        Some(config) => {
            let tls_config = ClientTlsConfig::new()
                .ca_certificate(config.ca_cert)
                .identity(config.identity);
            Endpoint::new(endpoint)?.tls_config(tls_config)?
        }
        None => Endpoint::new(endpoint)?,
    };
    let mut stage_client = StageServiceClient::connect(endpoint).await?;
    let response = stage_client.generate_proof(request).await?.into_inner();
    println!("response: {:?}", response);
    let end = Instant::now();
    let elapsed = end.duration_since(start);
    println!("Elapsed time: {:?} secs", elapsed.as_secs());
    Ok(())
}
