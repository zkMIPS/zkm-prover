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
    input_dir: String,
    output_dir: String,
}

impl SnarkProver {
    pub fn new(input_dir: &str, output_dir: &str) -> Self {
        Self {
            input_dir: input_dir.to_string(),
            output_dir: output_dir.to_string(),
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

        //let input_dir = format!("{}/{}", self.input_dir, ctx.proof_id);
        let output_dir = format!("{}/{}", self.output_dir, ctx.proof_id);
        std::fs::create_dir_all(&output_dir)?;

        log::info!(
            "snark prove: input_dir {:?}, output_dir: {:?}",
            self.input_dir,
            output_dir
        );

        assert!(ctx.agg_receipt.len() > 0);
        // wrap stark
        let all_circuits = &*crate::provers::instance().lock().unwrap();
        let agg_receipt = serde_json::from_slice(&ctx.agg_receipt)?;
        zkm_recursion::wrap_stark_bn254(all_circuits, agg_receipt, &self.input_dir)?;

        as_groth16(&self.input_dir, &self.input_dir, &output_dir)?;

        let snark_proof_with_public_inputs = std::fs::read(&format!(
            "{}/snark_proof_with_public_inputs.json",
            output_dir
        ))?;

        Ok((true, snark_proof_with_public_inputs))
    }
}
