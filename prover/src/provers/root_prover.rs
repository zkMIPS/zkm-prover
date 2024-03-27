use super::Prover;
use crate::contexts::ProveContext;

use num::ToPrimitive;
use std::time::Duration;

use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::plonk::config::PoseidonGoldilocksConfig;
use plonky2::util::timing::TimingTree;

use zkm::all_stark::AllStark;
use zkm::config::StarkConfig;
use zkm::cpu::kernel::assembler::segment_kernel;
use zkm::fixed_recursive_verifier::AllRecursiveCircuits;

use std::fs::File;
use std::io::Write;

#[derive(Default)]
pub struct RootProver {}

impl RootProver {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Prover<ProveContext> for RootProver {
    fn prove(&self, ctx: &ProveContext) -> anyhow::Result<()> {
        type F = GoldilocksField;
        const D: usize = 2;
        type C = PoseidonGoldilocksConfig;

        let basedir = ctx.basedir.clone();
        let block_no = ctx.block_no.to_string();
        let seg_path = ctx.seg_path.clone();
        let seg_size = ctx.seg_size.to_usize().expect("u32->usize failed");
        let proof_path = ctx.proof_path.clone();
        let pub_value_path = ctx.pub_value_path.clone();
        let file = String::from("");
        let _args = "".to_string();

        let all_stark = AllStark::<F, D>::default();
        let config = StarkConfig::standard_fast_config();
        // Preprocess all circuits.
        let all_circuits = AllRecursiveCircuits::<F, C, D>::new(
            &all_stark,
            // &[10..21, 15..22, 14..21, 9..21, 12..21, 15..23],
            &[10..21, 12..22, 13..21, 8..21, 10..21, 13..23],
            &config,
        );

        let input = segment_kernel(&basedir, &block_no, &file, &seg_path, seg_size);
        let mut timing: TimingTree = TimingTree::new("prove root", log::Level::Info);
        let (agg_proof, updated_agg_public_values) =
            all_circuits.prove_root(&all_stark, &input, &config, &mut timing)?;

        timing.filter(Duration::from_millis(100)).print();
        all_circuits.verify_root(agg_proof.clone())?;

        // write agg_proof write file
        let json_string = serde_json::to_string(&agg_proof)?;
        let mut file = File::create(proof_path)?;
        file.write_all(json_string.as_bytes())?;
        file.flush()?;

        // updated_agg_public_values file
        let json_string = serde_json::to_string(&updated_agg_public_values)?;
        let mut file = File::create(pub_value_path)?;
        file.write_all(json_string.as_bytes())?;
        file.flush()?;

        Ok(())
    }
}
