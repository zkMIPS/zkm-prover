use common::file;
use common::tls::Config;
use std::env;
use std::path::Path;
use std::time::Instant;
use tokio::time;
use tonic::transport::ClientTlsConfig;
use tonic::transport::Endpoint;

use ethers::signers::{LocalWallet, Signer};

use proof_service::proto::includes::v1::BlockFileItem;
use proof_service::proto::stage_service::v1::Status;
use proof_service::proto::stage_service::v1::{
    stage_service_client::StageServiceClient, GenerateProofRequest, GetStatusRequest,
};

async fn sign_ecdsa(request: &mut GenerateProofRequest, private_key: &str) {
    assert!(!private_key.is_empty());
    if !private_key.is_empty() {
        let wallet = private_key.parse::<LocalWallet>().unwrap();
        let sign_data = match request.block_no {
            Some(block_no) => {
                format!("{}&{}&{}", request.proof_id, block_no, request.seg_size)
            }
            None => {
                format!("{}&{}", request.proof_id, request.seg_size)
            }
        };
        let signature = wallet.sign_message(sign_data).await.unwrap();
        request.signature = signature.to_string();
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // first wallet in hardhat node
    env::set_var(
        "PRIVATE_KEY",
        "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
    );

    env::set_var(
        "ARGS",
        "711e9609339e92b03ddc0a211827dba421f38f9ed8b9d806e1ffdd8c15ffa03d world!",
    );

    env_logger::try_init().unwrap_or_default();
    let elf_path = env::var("ELF_PATH").unwrap_or("/tmp/zkm/test/hello_world".to_string());
    let output_dir = env::var("OUTPUT_DIR").unwrap_or("/tmp/zkm/test".to_string());
    let block_path = env::var("BLOCK_PATH").unwrap_or("".to_string());
    let block_no = env::var("BLOCK_NO").unwrap_or("0".to_string());
    let block_no = block_no.parse::<_>().unwrap_or(0);
    let seg_size = env::var("SEG_SIZE").unwrap_or("131072".to_string());
    let seg_size = seg_size.parse::<_>().unwrap_or(131072);
    let args = env::var("ARGS").unwrap_or("".to_string());
    let _public_input_path = env::var("PUBLIC_INPUT_PATH").unwrap_or("".to_string());
    let _private_input_path = env::var("PRIVATE_INPUT_PATH").unwrap_or("".to_string());
    let endpoint = env::var("ENDPOINT").unwrap_or("http://127.0.0.1:50000".to_string());
    let ca_cert_path = env::var("CA_CERT_PATH").unwrap_or("".to_string());
    let cert_path = env::var("CERT_PATH").unwrap_or("".to_string());
    let key_path = env::var("KEY_PATH").unwrap_or("".to_string());
    let domain_name = env::var("DOMAIN_NAME").unwrap_or("stage".to_string());
    let private_key = env::var("PRIVATE_KEY").unwrap_or("".to_string());
    let target_step = env::var("TARGET_STEP").unwrap_or("5".to_string());
    let target_step = target_step.parse::<i32>().unwrap_or(5);
    let ssl_config = if ca_cert_path.is_empty() {
        None
    } else {
        Some(Config::new(&ca_cert_path, &cert_path, &key_path).await?)
    };

    let elf_data = file::new(&elf_path).read().unwrap();

    // FIXME
    let mut block_data = Vec::new();
    if block_no > 0 {
        let files = file::new(&block_path).read_dir().unwrap();
        for file_name in files {
            let file_path = format!("{}/{}", block_path, file_name);
            let block_file_item = BlockFileItem {
                file_name: file_name.to_string(),
                file_content: file::new(&file_path).read().unwrap(),
            };
            block_data.push(block_file_item);
        }
    }

    let args: Vec<&str> = args.split_whitespace().collect();
    // let public_input_stream: Vec<u8> = hex::decode(args[0]).unwrap();
    let private_input_stream = args[1].as_bytes().to_vec();

    // It depends on whether the guest program uses io::read() or io::read_vec().
    // If it’s the former, then `bincode::serialize` is used; otherwise, it’s not.
    // In `sha2-rust` example, the guest program uses io::read().
    // let public_input_stream: Vec<u8> = bincode::serialize(&public_input_stream).unwrap();
    let private_input_stream: Vec<u8> = bincode::serialize(&private_input_stream).unwrap();
    let private_input_stream: Vec<u8> = bincode::serialize(&vec![private_input_stream]).unwrap();

    //let public_input_stream = if public_input_path.is_empty() {
    //    vec![]
    //} else {
    //    file::new(&public_input_path).read().unwrap()
    //};

    //let private_input_stream = if private_input_path.is_empty() {
    //    vec![]
    //} else {
    //    file::new(&private_input_path).read().unwrap()
    //};

    let proof_id = uuid::Uuid::new_v4().to_string();
    let mut request = GenerateProofRequest {
        proof_id: proof_id.clone(),
        elf_data,
        block_data,
        block_no: Some(block_no),
        seg_size,
        public_input_stream: vec![],
        private_input_stream,
        target_step: Some(target_step), // 1, 3, 5
        ..Default::default()
    };
    sign_ecdsa(&mut request, &private_key).await;
    log::info!("request: {:?}", proof_id);
    let start = Instant::now();
    let endpoint = match ssl_config {
        Some(config) => {
            let mut tls_config = ClientTlsConfig::new().domain_name(domain_name);
            if let Some(ca_cert) = config.ca_cert {
                tls_config = tls_config.ca_certificate(ca_cert);
            }
            if let Some(identity) = config.identity {
                tls_config = tls_config.identity(identity);
            }
            Endpoint::new(endpoint)?.tls_config(tls_config)?
        }
        None => Endpoint::new(endpoint)?,
    };
    let mut stage_client = StageServiceClient::connect(endpoint).await?;
    let response = stage_client.generate_proof(request).await?.into_inner();
    log::info!("generate_proof response: {:?}", response);
    if response.status == Status::Computing as i32 {
        loop {
            let get_status_request = GetStatusRequest {
                proof_id: proof_id.clone(),
            };
            let get_status_response = stage_client
                .get_status(get_status_request)
                .await?
                .into_inner();
            if get_status_response.status != Status::Computing as i32 {
                if let Some(status) = Status::from_i32(get_status_response.status) {
                    match status {
                        Status::Success => {
                            log::info!(
                                "generate_proof success public_inputs_size: {}, output_size: {}",
                                get_status_response.proof_with_public_inputs.len(),
                                get_status_response.output_stream.len(),
                            );
                            log::info!("public_values {:?}", get_status_response.public_values_url);
                            log::info!("snark_proof_url {:?}", get_status_response.snark_proof_url);
                            let output_dir = Path::new(&output_dir);
                            let public_inputs_path = output_dir.join("proof_with_public_inputs");
                            let _ = file::new(&public_inputs_path.to_string_lossy())
                                .write(get_status_response.proof_with_public_inputs.as_slice());
                        }
                        _ => {
                            log::info!(
                                "generate_proof failed status: {}",
                                get_status_response.status
                            );
                        }
                    }
                }
                break;
            }
            time::sleep(time::Duration::from_secs(1)).await;
        }
    }
    let end = Instant::now();
    let elapsed = end.duration_since(start);
    log::info!("Elapsed time: {:?} secs", elapsed.as_secs());
    Ok(())
}
