use super::Prover;
use crate::contexts::ProveContext;
use std::time::Duration;

use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::plonk::config::PoseidonGoldilocksConfig;
use plonky2::util::timing::TimingTree;

use zkm_prover::all_stark::AllStark;
use zkm_prover::config::StarkConfig;
use zkm_prover::cpu::kernel::assembler::segment_kernel;
use zkm_prover::generation::state::{AssumptionReceipts, Receipt};

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
        type C = PoseidonGoldilocksConfig;

        //let basedir = ctx.base_dir.clone();
        let block_no = ctx.block_no.unwrap_or(0);
        let segment = ctx.segment.clone();
        let receipt_path = ctx.receipt_path.clone();
        let file = String::from("");

        let mut receipts: AssumptionReceipts<F, C, D> = vec![];
        if !ctx.receipts_path.is_empty() {
            let data = file::new(&ctx.receipts_path)
                .read()
                .expect("read receipts_path failed");
            let receipt_datas =
                bincode::deserialize::<Vec<Vec<u8>>>(&data).expect("deserialize receipts failed");
            for receipt_data in receipt_datas.iter() {
                let receipt: Receipt<F, C, D> =
                    bincode::deserialize(receipt_data).map_err(|e| anyhow::anyhow!(e))?;
                receipts.push(receipt.into());
                log::info!("prove set receipts {:?}", receipt_data.len());
            }
        }

        let mut timing = TimingTree::new("root_prove init all_stark", log::Level::Info);
        let all_stark = AllStark::<F, D>::default();
        let config = StarkConfig::standard_fast_config();
        timing.filter(Duration::from_millis(100)).print();
        timing = TimingTree::new("root_prove init all_circuits", log::Level::Info);
        let all_circuits = &*crate::provers::instance().lock().unwrap();
        timing.filter(Duration::from_millis(100)).print();

        timing = TimingTree::new("root_prove load input", log::Level::Info);

        //let seg_data = file::new(&seg_path).read()?;
        let seg_reader = std::io::Cursor::new(segment);
        // FIXME: why do we need basedir?
        let input = segment_kernel("", &block_no.to_string(), &file, seg_reader);
        timing.filter(Duration::from_millis(100)).print();

        timing = TimingTree::new("root_prove root", log::Level::Info);
        let receipt = all_circuits.prove_root_with_assumption(
            &all_stark,
            &input,
            &config,
            &mut timing,
            receipts,
        )?;
        all_circuits.verify_root(receipt.clone())?;
        timing.filter(Duration::from_millis(100)).print();

        timing = TimingTree::new("root_prove write result", log::Level::Info);

        // write receipt write file
        let json_string = serde_json::to_string(&receipt)?;
        let _ = file::new(&receipt_path).write(json_string.as_bytes())?;
        timing.filter(Duration::from_millis(100)).print();

        Ok(())
    }
}
