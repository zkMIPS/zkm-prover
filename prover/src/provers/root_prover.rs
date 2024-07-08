use super::Prover;
use crate::contexts::ProveContext;
use std::time::Duration;

use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::util::timing::TimingTree;

use std::io::BufReader;
use zkm_prover::all_stark::AllStark;
use zkm_prover::config::StarkConfig;
use zkm_prover::cpu::kernel::assembler::segment_kernel;

use common::file;

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
        // type C = PoseidonGoldilocksConfig;

        let basedir = ctx.basedir.clone();
        let block_no = ctx.block_no.to_string();
        let seg_path = ctx.seg_path.clone();
        let seg_size = ctx.seg_size as usize;
        let proof_path = ctx.proof_path.clone();
        let pub_value_path = ctx.pub_value_path.clone();
        let file = String::from("");
        let _args = "".to_string();

        let mut timing = TimingTree::new("root_prove init all_stark", log::Level::Info);
        let all_stark = AllStark::<F, D>::default();
        let config = StarkConfig::standard_fast_config();
        timing.filter(Duration::from_millis(100)).print();
        timing = TimingTree::new("root_prove init all_circuits", log::Level::Info);
        let all_circuits = &*crate::provers::instance().lock().unwrap();
        timing.filter(Duration::from_millis(100)).print();

        timing = TimingTree::new("root_prove load input", log::Level::Info);

        let seg_data = file::new(&seg_path).read()?;
        let seg_reader = BufReader::new(seg_data.as_slice());
        let input = segment_kernel(&basedir, &block_no, &file, seg_reader, seg_size);
        timing.filter(Duration::from_millis(100)).print();

        timing = TimingTree::new("root_prove root", log::Level::Info);
        let (agg_proof, updated_agg_public_values) =
            all_circuits.prove_root(&all_stark, &input, &config, &mut timing)?;

        all_circuits.verify_root(agg_proof.clone())?;
        timing.filter(Duration::from_millis(100)).print();

        timing = TimingTree::new("root_prove write result", log::Level::Info);
        // write agg_proof write file
        let json_string = serde_json::to_string(&agg_proof)?;
        let _ = file::new(&proof_path).write(json_string.as_bytes())?;

        // updated_agg_public_values file
        let json_string = serde_json::to_string(&updated_agg_public_values)?;
        let _ = file::new(&pub_value_path).write(json_string.as_bytes())?;
        timing.filter(Duration::from_millis(100)).print();

        Ok(())
    }
}
