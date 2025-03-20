use zkm2_core_executor::ZKMReduceProof;
use zkm2_core_machine::ZKM_CIRCUIT_VERSION;
use zkm2_core_machine::io::ZKMStdin;
use zkm2_prover::{Groth16Bn254Proof, InnerSC, ZKMProver, ZKMRecursionProverError};
use zkm2_sdk::{ZKMProof, ZKMProofWithPublicValues};
use zkm2_stark::ZKMProverOpts;

use crate::NetworkProve;
use crate::contexts::SnarkContext;

#[derive(Default)]
pub struct SnarkProver {
    input_dir: String,
    output_dir: String,
}

impl SnarkProver {
    pub fn new(input_dir: &str, output_dir: &str) -> Self {
        Self {
            input_dir: input_dir.into(),
            output_dir: output_dir.into(),
        }
    }
    pub fn prove(&self, ctx: &SnarkContext) -> anyhow::Result<(bool, Vec<u8>)> {
        let reduced_proof: ZKMReduceProof<InnerSC> = bincode::deserialize(&ctx.agg_receipt)?;
        let network_prove = NetworkProve::new();

        let gnark_proof =
            self.prove_groth16(&network_prove.prover, reduced_proof, network_prove.opts)?;

        let output_dir = format!("{}/{}", self.output_dir, ctx.proof_id);
        std::fs::create_dir_all(&output_dir)?;

        let json_output = serde_json::to_string_pretty(&gnark_proof)?;
        std::fs::write(
            &format!("{}/gnark_proof_with_pis.json", output_dir),
            json_output,
        )?;

        tracing::info!(
            "snark prove: input_dir {:?}, output_dir: {:?}, gnark_proof file: gnark_proof_with_pis.json",
            self.input_dir,
            output_dir
        );

        Ok((true, bincode::serialize(&gnark_proof)?))
    }

    fn prove_groth16(
        &self,
        prover: &ZKMProver,
        reduced_proof: ZKMReduceProof<InnerSC>,
        opts: ZKMProverOpts,
    ) -> Result<ZKMProof, ZKMRecursionProverError> {
        let compress_proof = prover.shrink(reduced_proof, opts)?;
        let outer_proof = prover.wrap_bn254(compress_proof, opts)?;

        // TODO: use production artifacts `try_install_circuit_artifacts`
        let groth16_bn254_artifacts = if zkm2_prover::build::zkm2_dev_mode() {
            zkm2_prover::build::try_build_groth16_bn254_artifacts_dev(
                &outer_proof.vk,
                &outer_proof.proof,
            )
        } else {
            todo!("impl production artifacts");
            // try_install_circuit_artifacts("groth16")
        };
        let proof = prover.wrap_groth16_bn254(outer_proof, &groth16_bn254_artifacts);

        Ok(ZKMProof::Groth16(proof))
    }
}
