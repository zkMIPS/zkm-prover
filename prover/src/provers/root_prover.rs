use super::Prover;
use super::{C, D, F};
use crate::contexts::ProveContext;
use plonky2::util::timing::TimingTree;
use std::time::Duration;
use zkm_prover::all_stark::AllStark;
use zkm_prover::config::StarkConfig;
use zkm_prover::cpu::kernel::assembler::segment_kernel;
use zkm_prover::generation::state::{AssumptionReceipts, Receipt};
use common::file;

#[derive(Default)]
pub struct RootProver {}

impl Prover<ProveContext, Vec<u8>> for RootProver {
    fn prove(&self, ctx: &ProveContext) -> anyhow::Result<(bool, Vec<u8>)> {
        //let basedir = ctx.base_dir.clone();
        let block_no = ctx.block_no.unwrap_or(0);

        let mut receipts: AssumptionReceipts<F, C, D> = vec![];
        if !ctx.receipts_input.is_empty() {
            //let receipt_datas =
            //    bincode::deserialize::<Vec<Vec<u8>>>(&ctx.receipts_input).expect("deserialize receipts failed");
            for receipt_data in ctx.receipts_input.iter() {
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

        //let file = String::from("");
        let seg_data = file::new(&ctx.segment).read()?;
        // TODO: don't support block_data
        let seg_reader = std::io::Cursor::new(seg_data);
        let input = segment_kernel("", &block_no.to_string(), "", seg_reader);
        timing.filter(Duration::from_millis(100)).print();

        timing = TimingTree::new("root_prove prove", log::Level::Info);
        let receipt = all_circuits.prove_root_with_assumption(
            &all_stark,
            &input,
            &config,
            &mut timing,
            receipts,
        )?;
        all_circuits.verify_root(receipt.clone())?;
        timing.filter(Duration::from_millis(100)).print();

        //timing = TimingTree::new("root_prove write result", log::Level::Info);

        //// write receipt write file
        //let json_string = serde_json::to_string(&receipt)?;
        //let _ = file::new(&receipt_path).write(json_string.as_bytes())?;
        //timing.filter(Duration::from_millis(100)).print();
        Ok((true, serde_json::to_vec(&receipt)?))
    }
}
