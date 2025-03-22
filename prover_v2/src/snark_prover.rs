use std::path::PathBuf;
use zkm2_core_executor::ZKMReduceProof;
use zkm2_prover::{InnerSC, ZKMProver, ZKMRecursionProverError};
use zkm2_sdk::ZKMProof;
use zkm2_stark::ZKMProverOpts;

use crate::NetworkProve;
use crate::contexts::SnarkContext;

// It seems we don't need `output_dir`.
#[derive(Default)]
pub struct SnarkProver {
    proving_key_paths: String,
    output_dir: String,
}

impl SnarkProver {
    pub fn new(proving_key_paths: &str, output_dir: &str) -> Self {
        Self {
            proving_key_paths: proving_key_paths.into(),
            output_dir: output_dir.into(),
        }
    }
    pub fn prove(&self, ctx: &SnarkContext) -> anyhow::Result<(bool, Vec<u8>)> {
        let reduced_proof: ZKMReduceProof<InnerSC> = bincode::deserialize(&ctx.agg_receipt)?;
        let network_prove = NetworkProve::new();

        let gnark_proof =
            self.prove_groth16(&network_prove.prover, reduced_proof, network_prove.opts)?;

        Ok((true, serde_json::to_vec(&gnark_proof)?))
    }

    fn prove_groth16(
        &self,
        prover: &ZKMProver,
        reduced_proof: ZKMReduceProof<InnerSC>,
        opts: ZKMProverOpts,
    ) -> Result<ZKMProof, ZKMRecursionProverError> {
        let compress_proof = prover.shrink(reduced_proof, opts)?;
        let outer_proof = prover.wrap_bn254(compress_proof, opts)?;

        // TODO: Pull artifacts from the server
        // let groth16_bn254_artifacts = if zkm2_prover::build::zkm2_dev_mode() {
        //     zkm2_prover::build::try_build_groth16_bn254_artifacts_dev(
        //         &outer_proof.vk,
        //         &outer_proof.proof,
        //     )
        // } else {
        //     todo!("impl production artifacts");
        //     // try_install_circuit_artifacts("groth16")
        // };
        let groth16_bn254_artifacts = PathBuf::from(&self.proving_key_paths);
        let proof = prover.wrap_groth16_bn254(outer_proof, &groth16_bn254_artifacts);

        Ok(ZKMProof::Groth16(proof))
    }
}
