use zkm2_core_executor::ZKMReduceProof;
use zkm2_core_machine::ZKM_CIRCUIT_VERSION;
use zkm2_core_machine::io::ZKMStdin;
use zkm2_prover::{Groth16Bn254Proof, InnerSC, ZKMProver, ZKMRecursionProverError};
use zkm2_sdk::{ZKMProof, ZKMProofWithPublicValues};
use zkm2_stark::ZKMProverOpts;
use crate::contexts::snark_context::SnarkContext;
use crate::NetworkProve;

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
        let zkm_proof_with_public_values: ZKMProofWithPublicValues =
            bincode::deserialize(&ctx.agg_receipt)?;
        let network_prove = NetworkProve::new();

        let gnark_proof_with_pis = self.prove_groth16(
            &network_prove.prover,
            zkm_proof_with_public_values,
            network_prove.opts,
        )?;

        Ok((true, bincode::serialize(&gnark_proof_with_pis)?))
    }

    fn prove_groth16(
        &self,
        prover: &ZKMProver,
        zkm_proof_with_public_values: ZKMProofWithPublicValues,
        opts: ZKMProverOpts,
    ) -> Result<ZKMProofWithPublicValues, ZKMRecursionProverError> {
        let reduced_proof =
            if let ZKMProof::Compressed(reduced_proof) = zkm_proof_with_public_values.proof {
                *reduced_proof
            } else {
                return Err(ZKMRecursionProverError::RuntimeError(
                    "invalid input proof".into(),
                ));
            };

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

        Ok(ZKMProofWithPublicValues {
            proof: ZKMProof::Groth16(proof),
            stdin: zkm_proof_with_public_values.stdin,
            public_values: zkm_proof_with_public_values.public_values,
            zkm2_version: zkm_proof_with_public_values.zkm2_version,
        })
    }
}
