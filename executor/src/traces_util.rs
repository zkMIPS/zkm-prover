use std::time::Duration;

use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::plonk::config::PoseidonGoldilocksConfig;
use plonky2::util::timing::TimingTree;

use std::io::BufReader;
use zkm_prover::all_stark::AllStark;
use zkm_prover::config::StarkConfig;
use zkm_prover::cpu::kernel::assembler::segment_kernel;
use zkm_prover::generation;
use zkm_prover::generation::state::AssumptionReceipts;

use common::file;

use crate::split_context::SplitContext;

#[derive(Default)]
pub struct TracesUtil {}

impl TracesUtil {
    pub fn new() -> Self {
        Self::default()
    }
}

type F = GoldilocksField;
const D: usize = 2;
type C = PoseidonGoldilocksConfig;

impl TracesUtil {
    pub fn get_traces_len(
        ctx: &SplitContext,
        receipts: AssumptionReceipts<F, C, D>,
        seg_file: &str,
    ) -> anyhow::Result<Vec<usize>> {
        let basedir = ctx.basedir.clone();
        let block_no = ctx.block_no.to_string();
        let file = String::from("");

        let mut timing = TimingTree::new("generate_traces init all_stark", log::Level::Info);
        let all_stark = AllStark::<F, D>::default();
        let config = StarkConfig::standard_fast_config();
        timing.filter(Duration::from_millis(100)).print();

        timing = TimingTree::new("generate_traces load input", log::Level::Info);

        let seg_data = file::new(seg_file).read()?;
        let seg_reader = BufReader::new(seg_data.as_slice());
        let input = segment_kernel(&basedir, &block_no, &file, seg_reader);
        timing.filter(Duration::from_millis(100)).print();

        timing = TimingTree::new("generate_traces", log::Level::Info);
        // let trace_meta = generation::trace_check_point::<F, C, D>(&input, &mut timing, receipts)?;
        let mut trace_meta = Vec::new();
        if receipts.is_empty() {
            let (traces, _, _) =
                generation::generate_traces::<F, C, D>(&all_stark, &input, &config, &mut timing)?;
            traces.iter().for_each(|trace| {
                trace_meta.push(trace[0].len());
            });
        } else {
            let (traces, _, _, _) = generation::generate_traces_with_assumptions::<F, C, D>(
                &all_stark,
                &input,
                &config,
                &mut timing,
                receipts,
            )?;
            traces.iter().for_each(|trace| {
                trace_meta.push(trace[0].len());
            });
        }
        timing.filter(Duration::from_millis(100)).print();

        Ok(trace_meta)
    }
}
