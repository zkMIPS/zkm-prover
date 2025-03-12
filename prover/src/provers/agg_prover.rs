use super::Prover;
use crate::contexts::AggContext;
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::plonk::config::PoseidonGoldilocksConfig;
use plonky2::util::timing::TimingTree;
use std::time::Duration;

use zkm_prover::generation::state::Receipt;

#[derive(Default)]
pub struct AggProver {}

impl Prover<AggContext, Vec<u8>> for AggProver {
    fn prove(&self, ctx: &AggContext) -> anyhow::Result<(bool, Vec<u8>)> {
        type F = GoldilocksField;
        const D: usize = 2;
        type C = PoseidonGoldilocksConfig;

        let receipt_path1 = ctx.receipt_path1.clone();
        let receipt_path2 = ctx.receipt_path2.clone();
        //let agg_receipt_path = ctx.agg_receipt_path.clone();
        let is_agg1 = ctx.is_agg_1;
        let is_agg2 = ctx.is_agg_2;

        let mut timing = TimingTree::new("agg init all_circuits", log::Level::Info);
        let all_circuits = &*crate::provers::instance().lock().unwrap();
        timing.filter(Duration::from_millis(100)).print();

        //let receipt_first_content = file::new(&receipt_path1).read_to_string()?;
        let receipt_first: Receipt<F, C, D> = serde_json::from_slice(&receipt_path1)?;

        //let receipt_next_content = file::new(&receipt_path2).read_to_string()?;
        let receipt_next: Receipt<F, C, D> = serde_json::from_slice(&receipt_path2)?;

        // We can duplicate the proofs here because the state hasn't mutated.
        let new_agg_receipt = zkm_recursion::aggregate_proof(
            all_circuits,
            receipt_first,
            receipt_next,
            is_agg1,
            is_agg2,
        )?;
        // write receipt write file
        let agg_receipt_output = serde_json::to_vec(&new_agg_receipt)?;
        //let _ = file::new(&agg_receipt_path).write(json_string.as_bytes())?;

        Ok((true, agg_receipt_output))
    }
}
