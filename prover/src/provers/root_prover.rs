use super::Prover;
use crate::contexts::ProveContext;
use crate::provers::select_degree_bits;
use std::time::Duration;

use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::plonk::config::PoseidonGoldilocksConfig;
use plonky2::util::timing::TimingTree;

use zkm::all_stark::AllStark;
use zkm::config::StarkConfig;
use zkm::cpu::kernel::assembler::segment_kernel;
use zkm::fixed_recursive_verifier::AllRecursiveCircuits;

use common::file::write_file;

#[derive(Default)]
pub struct RootProver {}

impl RootProver {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Prover<ProveContext> for RootProver {
    async fn prove(&self, ctx: &ProveContext) -> anyhow::Result<()> {
        type F = GoldilocksField;
        const D: usize = 2;
        type C = PoseidonGoldilocksConfig;

        let basedir = ctx.basedir.clone();
        let block_no = ctx.block_no.to_string();
        let seg_path = ctx.seg_path.clone();
        let seg_size = ctx.seg_size as usize;
        let proof_path = ctx.proof_path.clone();
        let pub_value_path = ctx.pub_value_path.clone();
        let file = String::from("");
        let _args = "".to_string();

        let all_stark = AllStark::<F, D>::default();
        let config = StarkConfig::standard_fast_config();
        // Preprocess all circuits.
        let all_circuits = AllRecursiveCircuits::<F, C, D>::new(
            &all_stark,
            &select_degree_bits(seg_size),
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
        write_file(&proof_path, json_string.as_bytes()).await?;

        // updated_agg_public_values file
        let json_string = serde_json::to_string(&updated_agg_public_values)?;
        write_file(&pub_value_path, json_string.as_bytes()).await?;

        Ok(())
    }
}
