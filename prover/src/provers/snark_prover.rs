use crate::provers::Prover;
use zkm_recursion::as_groth16;
use crate::contexts::SnarkContext;

#[derive(Default)]
pub struct SnarkProver {}

impl Prover<SnarkContext> for SnarkProver {
    fn prove(&self, ctx: &mut SnarkContext) -> anyhow::Result<()> {
        as_groth16(&ctx.pk_dir, &ctx.input_dir, &ctx.output_dir)?;
        Ok(())
    }
}