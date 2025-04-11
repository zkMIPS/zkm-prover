use crate::contexts::SnarkContext;
use crate::provers::Prover;
use zkm_recursion::as_groth16;

#[derive(Default)]
pub struct SnarkProver {
    key_path: String,
    base_dir: String,
}

impl SnarkProver {
    pub fn new(key_path: &str, base_dir: &str) -> Self {
        Self {
            key_path: key_path.to_string(),
            base_dir: base_dir.to_string(),
        }
    }
}

impl Prover<SnarkContext, Vec<u8>> for SnarkProver {
    fn prove(&self, ctx: &SnarkContext) -> anyhow::Result<(bool, Vec<u8>)> {
        let base_dir = format!("{}/{}", self.base_dir, ctx.proof_id);
        let input_dir = format!("{}/wrap", base_dir);
        let output_dir = format!("{}/snark", base_dir);
        std::fs::create_dir_all(&input_dir)?;
        std::fs::create_dir_all(&output_dir)?;

        log::info!(
            "snark prove: input_dir {:?}, output_dir: {:?}",
            input_dir,
            output_dir
        );

        assert!(!ctx.agg_receipt.is_empty());
        // wrap stark
        let all_circuits = &*crate::provers::instance().lock().unwrap();
        let agg_receipt = serde_json::from_slice(&ctx.agg_receipt)?;
        zkm_recursion::wrap_stark_bn254(all_circuits, agg_receipt, &input_dir)?;

        as_groth16(&self.key_path, &input_dir, &output_dir)?;

        let snark_proof_with_public_inputs = std::fs::read(format!(
            "{}/snark_proof_with_public_inputs.json",
            output_dir
        ))?;

        Ok((true, snark_proof_with_public_inputs))
    }
}
