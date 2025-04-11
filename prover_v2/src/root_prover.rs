use zkm_core_executor::ExecutionRecord;
use zkm_stark::{MachineProver, MachineProvingKey, StarkGenericConfig};

use crate::contexts::ProveContext;
use crate::{get_prover, NetworkProve};

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
        let network_prove = NetworkProve::new();
        let opts = network_prove.opts.core_opts;

        let prover = get_prover();
        let (pk, _) = prover.core_prover.machine().setup(&record.program);
        prover.core_prover
            .machine()
            .generate_dependencies(std::slice::from_mut(&mut record), &opts, None);

        // Fix the shape of the record.
        if let Some(shape_config) = &prover.core_shape_config {
            shape_config.fix_shape(&mut record).unwrap();
        }
        let main_trace = prover.core_prover.generate_traces(&record);

        let mut challenger = prover.core_prover.config().challenger();
        pk.observe_into(&mut challenger);
        let main_data = prover.core_prover.commit(&record, main_trace);
        let proof = prover.core_prover.open(&pk, main_data, &mut challenger)?;

        Ok(bincode::serialize(&proof)?)
    }
}
