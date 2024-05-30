use ethers::{
    contract::abigen,
    middleware::SignerMiddleware,
    providers::{Http, Provider},
    signers::{LocalWallet, Signer},
};

use ethers::types::Address;
use ethers::types::U256;

use std::env;
use std::{sync::Arc, time::Duration};

use serde_json::Value;
use std::fs::File;
use std::io::BufReader;

// Generate the type-safe contract bindings by providing the ABI
// definition
abigen!(Verifier, "./service/examples/Verifier.abi");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::try_init().unwrap_or_default();
    let chan_id = env::var("CHAIN_ID").unwrap_or("11155111".to_string());
    let chan_id: u32 = chan_id.parse::<_>().unwrap_or(1);
    let node_url = env::var("NODE_URL").unwrap_or(
        "https://eth-sepolia.g.alchemy.com/v2/RH793ZL_pQkZb7KttcWcTlOjPrN0BjOW".to_string(),
    );
    let private_key = env::var("PRIVATE_KEY").unwrap_or("".to_string());
    let contract_addr =
        env::var("CONTRACT_ADDR").unwrap_or("012ef3e31BA2664163bD039535889aE7bE9E7E86".to_string());
    let proof_path =
        env::var("PROOF_PATH").unwrap_or("/tmp/proof_with_public_input.json".to_string());

    let wallet = private_key.parse::<LocalWallet>().unwrap();
    let provider = Provider::<Http>::try_from(node_url)?.interval(Duration::from_millis(10000u64));

    let client = SignerMiddleware::new(provider, wallet.with_chain_id(chan_id));
    let client = Arc::new(client);

    let addr = Address::from_slice(&hex::decode(contract_addr.into_bytes()).unwrap());
    let contract = Verifier::new(addr, client.clone());

    let file = File::open(proof_path)?;
    let reader = BufReader::new(file);
    let data: Value = serde_json::from_reader(reader)?;

    let mut public_inputs: [U256; 65] = [U256::zero(); 65];
    let mut commitments_xy = [U256::zero(); 2];

    let mut index = 0;
    if let Some(list_data) = data.get("PublicWitness").and_then(|v| v.as_array()) {
        let list: Vec<String> = list_data
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
        for item in list {
            if index < 65 {
                public_inputs[index] = U256::from_dec_str(&item).unwrap();
            }
            index += 1;
        }
    }

    if let Some(proof) = data.get("Proof").and_then(|v| v.as_object()) {
        if proof.contains_key("Commitments") {
            if let Some(commitments) = proof["Commitments"].as_array() {
                if !commitments.is_empty() {
                    let commitments_x = commitments[0]
                        .as_object()
                        .unwrap()
                        .get("X")
                        .unwrap()
                        .as_str()
                        .unwrap();
                    let commitments_y = commitments[0]
                        .as_object()
                        .unwrap()
                        .get("Y")
                        .unwrap()
                        .as_str()
                        .unwrap();
                    commitments_xy = [
                        U256::from_dec_str(commitments_x).unwrap(),
                        U256::from_dec_str(commitments_y).unwrap(),
                    ];
                }
            }
        }
    }

    let ga = G1Point {
        x: U256::from_dec_str(
            data.get("Proof")
                .unwrap()
                .get("Ar")
                .unwrap()
                .get("X")
                .unwrap()
                .as_str()
                .unwrap(),
        )
        .unwrap(),
        y: U256::from_dec_str(
            data.get("Proof")
                .unwrap()
                .get("Ar")
                .unwrap()
                .get("Y")
                .unwrap()
                .as_str()
                .unwrap(),
        )
        .unwrap(),
    };
    let gb = G2Point {
        x: [
            U256::from_dec_str(
                data.get("Proof")
                    .unwrap()
                    .get("Bs")
                    .unwrap()
                    .get("X")
                    .unwrap()
                    .get("A0")
                    .unwrap()
                    .as_str()
                    .unwrap(),
            )
            .unwrap(),
            U256::from_dec_str(
                data.get("Proof")
                    .unwrap()
                    .get("Bs")
                    .unwrap()
                    .get("X")
                    .unwrap()
                    .get("A1")
                    .unwrap()
                    .as_str()
                    .unwrap(),
            )
            .unwrap(),
        ],
        y: [
            U256::from_dec_str(
                data.get("Proof")
                    .unwrap()
                    .get("Bs")
                    .unwrap()
                    .get("Y")
                    .unwrap()
                    .get("A0")
                    .unwrap()
                    .as_str()
                    .unwrap(),
            )
            .unwrap(),
            U256::from_dec_str(
                data.get("Proof")
                    .unwrap()
                    .get("Bs")
                    .unwrap()
                    .get("Y")
                    .unwrap()
                    .get("A1")
                    .unwrap()
                    .as_str()
                    .unwrap(),
            )
            .unwrap(),
        ],
    };
    let gc = G1Point {
        x: U256::from_dec_str(
            data.get("Proof")
                .unwrap()
                .get("Krs")
                .unwrap()
                .get("X")
                .unwrap()
                .as_str()
                .unwrap(),
        )
        .unwrap(),
        y: U256::from_dec_str(
            data.get("Proof")
                .unwrap()
                .get("Krs")
                .unwrap()
                .get("Y")
                .unwrap()
                .as_str()
                .unwrap(),
        )
        .unwrap(),
    };
    let proof = Proof {
        a: ga.clone(),
        b: gb,
        c: gc,
    };
    let gas_append = 1000;
    let method = contract.verify_tx(proof, public_inputs, commitments_xy);
    let gas_estimate = method.estimate_gas().await?;
    println!("Estimated Gas: {:?}", gas_estimate);
    let binding = method.gas(gas_estimate + gas_append);
    let tx = binding.send().await?;
    let tx_hash = tx.tx_hash();
    println!("Transaction hash: {:?}", tx_hash);
    let receipt = tx.await?;
    println!("receipt: {:?}", receipt);

    Ok(())
}
