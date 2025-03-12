use crate::contexts::SnarkContext;
use crate::provers::Prover;
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::plonk::config::PoseidonGoldilocksConfig;
use std::path::Path;
use zkm_recursion::as_groth16;

use plonky2x::backend::circuit::Groth16WrapperParameters;
use plonky2x::prelude::DefaultParameters;

#[derive(Default)]
pub struct SnarkProver {
    proving_key_path: String,
    input_dir: String,
    output_dir: String,
}

impl SnarkProver {
    pub fn new(proving_key_path: String, input_dir: String, output_dir: String) -> Self {
        if !Path::new(&proving_key_path).exists() {
            panic!("{} not exist", proving_key_path);
        }
        Self {
            proving_key_path,
            input_dir,
            output_dir,
        }
    }
}

impl Prover<SnarkContext, Vec<u8>> for SnarkProver {
    fn prove(&self, ctx: &SnarkContext) -> anyhow::Result<(bool, Vec<u8>)> {
        type InnerParameters = DefaultParameters;
        type OuterParameters = Groth16WrapperParameters;
        type F = GoldilocksField;
        const D: usize = 2;
        type C = PoseidonGoldilocksConfig;

        let input_dir = format!("{}/{}", self.input_dir, ctx.proof_id);
        std::fs::create_dir_all(&input_dir)?;

        // wrap stark
        let all_circuits = &*crate::provers::instance().lock().unwrap();
        let agg_receipt = serde_json::from_slice(&ctx.agg_receipt)?;
        zkm_recursion::wrap_stark_bn254(all_circuits, agg_receipt, &self.output_dir)?;

        let (
            common_circuit_data_file,
            verifier_only_circuit_data_file,
            proof_with_public_inputs_file,
            block_public_inputs_file,
        ) = {
            (
                format!("{}/common_circuit_data.json", input_dir),
                format!("{}/verifier_only_circuit_data.json", input_dir),
                format!("{}/proof_with_public_inputs.json", input_dir),
                format!("{}/block_public_inputs.json", input_dir),
            )
        };
        std::fs::write(common_circuit_data_file, &ctx.common_circuit_data)?;
        std::fs::write(
            verifier_only_circuit_data_file,
            &ctx.verifier_only_circuit_data,
        )?;
        std::fs::write(proof_with_public_inputs_file, &ctx.proof_with_public_inputs)?;
        std::fs::write(block_public_inputs_file, &ctx.block_public_inputs)?;

        let output_dir = format!("{}/{}", self.output_dir, ctx.proof_id);
        std::fs::create_dir_all(&input_dir)?;

        as_groth16(&self.proving_key_path, &input_dir, &output_dir)?;

        let snark_proof_with_public_inputs = std::fs::read(&format!(
            "{}/snark_proof_with_public_inputs.json",
            output_dir
        ))?;

        Ok((true, snark_proof_with_public_inputs))
    }
}
