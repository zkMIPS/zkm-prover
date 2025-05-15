use zkm_core_executor::ExecutionRecord;
use zkm_stark::{MachineProver, StarkGenericConfig};

use crate::contexts::ProveContext;
use crate::{get_prover, NetworkProve};

#[derive(Default)]
pub struct RootProver {}

impl RootProver {
    pub fn prove(&self, ctx: &ProveContext) -> anyhow::Result<Vec<u8>> {
        let now = std::time::Instant::now();
        let mut record: ExecutionRecord = {
            let mut retries = 0;
            const MAX_RETRIES: usize = 5;

            loop {
                match std::fs::read(&ctx.segment).and_then(|segment| {
                    bincode::deserialize(&segment)
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
                }) {
                    Ok(record) => break record,
                    Err(e) => {
                        if retries >= MAX_RETRIES {
                            return Err(e.into());
                        }
                        tracing::warn!("read segment file error, try again");
                        retries += 1;
                        std::thread::sleep(std::time::Duration::from_millis(200));
                    }
                }
            }
        };
        tracing::info!("read segment time: {:?}", now.elapsed());

        let network_prove = NetworkProve::new(ctx.seg_size);
        let opts = network_prove.opts.core_opts;

        let prover = get_prover();
        let now = std::time::Instant::now();
        let (pk, _) = prover.core_prover.machine().setup(&record.program);
        tracing::info!("setup time: {:?}", now.elapsed());
        let now = std::time::Instant::now();
        prover.core_prover.machine().generate_dependencies(
            std::slice::from_mut(&mut record),
            &opts,
            None,
        );
        tracing::info!("generate dependencies time: {:?}", now.elapsed());

        // Fix the shape of the record.
        let now = std::time::Instant::now();
        if let Some(shape_config) = &prover.core_shape_config {
            shape_config.fix_shape(&mut record).unwrap();
        }
        tracing::info!("fix shape time: {:?}", now.elapsed());
        let now = std::time::Instant::now();
        let main_trace = prover.core_prover.generate_traces(&record);
        tracing::info!("generate traces time: {:?}", now.elapsed());

        let mut challenger = prover.core_prover.config().challenger();
        pk.observe_into(&mut challenger);
        let now = std::time::Instant::now();
        let main_data = prover.core_prover.commit(&record, main_trace);
        tracing::info!("commit time: {:?}", now.elapsed());
        let now = std::time::Instant::now();
        let proof = prover.core_prover.open(&pk, main_data, &mut challenger)?;
        tracing::info!("open time: {:?}", now.elapsed());

        Ok(bincode::serialize(&proof)?)
    }
}
