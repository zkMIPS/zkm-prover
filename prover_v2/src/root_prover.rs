use zkm2_core_executor::ExecutionRecord;
use zkm2_stark::{MachineProver, MachineProvingKey, StarkGenericConfig};

use crate::contexts::ProveContext;
use crate::NetworkProve;

#[derive(Default)]
pub struct RootProver {}

impl RootProver {
    pub fn prove(&self, ctx: &ProveContext) -> anyhow::Result<Vec<u8>> {
        let segment = std::fs::read(&ctx.segment)?;
        let mut record: ExecutionRecord = bincode::deserialize(&segment)?;

        // set SHARD_SIZE
        if ctx.seg_size > 0 {
            std::env::set_var("SHARD_SIZE", ctx.seg_size.to_string());
        }
        let mut network_prove = NetworkProve::new();
        let opts = network_prove.opts.core_opts;
        let prover = network_prove.prover.core_prover;
        let (pk, _) = prover.machine().setup(&record.program);
        prover
            .machine()
            .generate_dependencies(std::slice::from_mut(&mut record), &opts, None);

        // Fix the shape of the record.
        if let Some(shape_config) = network_prove.prover.core_shape_config {
            shape_config.fix_shape(&mut record).unwrap();
        }
        let main_trace = prover.generate_traces(&record);

        let mut challenger = prover.config().challenger();
        pk.observe_into(&mut challenger);
        let main_data = prover.commit(&record, main_trace);
        let proof = prover.open(&pk, main_data, &mut challenger)?;

        Ok(bincode::serialize(&proof)?)
    }
}
