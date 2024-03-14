use stage_service::stage_service_client::StageServiceClient;
use stage_service::{ExecutorError, GenerateProofRequest};

pub mod stage_service {
    tonic::include_proto!("stage.v1");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let elf_data = prover::provers::read_file_bin(&"/tmp/zkm/test/hello_world".to_string()).unwrap();
    let block_data = prover::provers::read_file_bin(&"/tmp/zkm/test/0_13284491/input".to_string()).unwrap();

    let request = GenerateProofRequest {
        proof_id: uuid::Uuid::new_v4().to_string(),
        elf_data: elf_data,
        block_data: block_data,
        block_no: 13284491,
        seg_size: 262144,
        ..Default::default()
    };
    println!("request: {:?}", request.proof_id.clone());
    let mut stage_client = StageServiceClient::connect("http://127.0.0.1:50000").await?;
    let response = stage_client.generate_proof(request).await?.into_inner();
    println!("response: {:?}", response);
    Ok(())
}