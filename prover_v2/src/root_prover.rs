use p3_field::PrimeField32;
use p3_maybe_rayon::prelude::*;
use std::env;

use zkm2_core_executor::{ExecutionReport, ExecutionState, Program, ZKMContext};
use zkm2_core_machine::io::ZKMStdin;
use zkm2_core_machine::utils::{ZKMCoreProverError, chunk_vec, trace_checkpoint};
use zkm2_core_machine::{CoreShapeConfig, MipsAir};
use zkm2_prover::ZKMProver;
use zkm2_stark::{
    Com, MachineProof, MachineProver, MachineProvingKey, MachineRecord, OpeningProof,
    PcsProverData, ShardProof, StarkGenericConfig, ZKMCoreOpts,
};

use crate::NetworkProve;
use crate::contexts::ProveContext;
use crate::utils::concurrency::DistributedLock;

#[derive(Default)]
pub struct RootProver {}

impl RootProver {
    pub fn prove(&self, ctx: &ProveContext) -> anyhow::Result<Vec<Vec<u8>>> {
        let mut network_prove = NetworkProve::new();
        let program = network_prove
            .prover
            .get_program(&ctx.elf)
            .map_err(|e| anyhow::Error::msg(e.to_string()))?;
        let (pk, _) = network_prove.prover.core_prover.setup(&program);

        let state = bincode::deserialize(&ctx.segment)?;

        let redis = env::var("REDIS_URL").unwrap_or("http://127.0.0.1:6379".to_string());

        let (segments_proof, _report) = tokio::runtime::Runtime::new()?.block_on(async {
            let sync = DistributedLock::new(
                &redis,
                &format!("{}/turn", ctx.proof_id),
                &format!("{}/turn_channel", ctx.proof_id),
                &format!("{}/state_key", ctx.proof_id),
                &format!("{}/deferred_key", ctx.proof_id),
            ).await?;

            self.prove_segment(
                &network_prove.prover.core_prover,
                &pk,
                program,
                state,
                ctx.index,
                ctx.done,
                sync,
                network_prove.opts.core_opts,
                network_prove.prover.core_shape_config.as_ref(),
            ).await
        })?;
        let results = segments_proof.into_par_iter().map(|proof| bincode::serialize(&proof).expect("failed to serialize proof")).collect();

        Ok(results)
    }

    async fn prove_segment<SC: StarkGenericConfig, P: MachineProver<SC, MipsAir<SC::Val>>>(
        &self,
        prover: &P,
        pk: &P::DeviceProvingKey,
        program: Program,
        checkpoint: ExecutionState,
        index: usize,
        done: bool,
        sync: DistributedLock,
        opts: ZKMCoreOpts,
        shape_config: Option<&CoreShapeConfig<SC::Val>>,
    ) -> anyhow::Result<(Vec<ShardProof<SC>>, ExecutionReport)>
    where
        SC::Val: PrimeField32,
        SC::Challenger: 'static + Clone + Send,
        OpeningProof<SC>: Send,
        Com<SC>: Send + Sync,
        PcsProverData<SC>: Send + Sync,
    {
        // Trace the checkpoint and reconstruct the execution records.
        let (mut records, report) = tracing::debug_span!("trace checkpoint")
            .in_scope(|| trace_checkpoint::<SC>(program.clone(), checkpoint, opts, shape_config));
        tracing::debug!("generated {} records", records.len());
        // *report_aggregate.lock().unwrap() += report;
        // reset_seek(&mut checkpoint);

        // Wait for our turn to update the state.
        sync.wait_for_turn(index)
            .await
            .expect("failed to wait for turn");

        // Update the public values & prover state for the shards which contain
        // "cpu events".
        let mut state = sync.get_state().await.expect("failed to get state");
        for record in records.iter_mut() {
            state.shard += 1;
            state.execution_shard = record.public_values.execution_shard;
            state.start_pc = record.public_values.start_pc;
            state.next_pc = record.public_values.next_pc;
            state.committed_value_digest = record.public_values.committed_value_digest;
            state.deferred_proofs_digest = record.public_values.deferred_proofs_digest;
            record.public_values = state;
        }

        // Defer events that are too expensive to include in every shard.
        let mut deferred = sync.get_deferred().await.expect("failed to get deferred");
        for record in records.iter_mut() {
            deferred.append(&mut record.defer());
        }

        // See if any deferred shards are ready to be committed to.
        let mut deferred = deferred.split(done, opts.split_opts);
        tracing::debug!("deferred {} records", deferred.len());

        // Update the public values & prover state for the shards which do not
        // contain "cpu events" before committing to them.
        if !done {
            state.execution_shard += 1;
        }
        for record in deferred.iter_mut() {
            state.shard += 1;
            state.previous_init_addr_bits = record.public_values.previous_init_addr_bits;
            state.last_init_addr_bits = record.public_values.last_init_addr_bits;
            state.previous_finalize_addr_bits = record.public_values.previous_finalize_addr_bits;
            state.last_finalize_addr_bits = record.public_values.last_finalize_addr_bits;
            state.start_pc = state.next_pc;
            record.public_values = state;
        }
        records.append(&mut deferred);

        // Generate the dependencies.
        tracing::debug_span!("generate dependencies", index).in_scope(|| {
            prover
                .machine()
                .generate_dependencies(&mut records, &opts, None);
        });

        // Let another worker update the state.
        sync.advance_turn().await.expect("failed to advance turn");

        // Fix the shape of the records.
        if let Some(shape_config) = shape_config {
            for record in records.iter_mut() {
                shape_config.fix_shape(record).unwrap();
            }
        }

        let mut main_traces = Vec::new();
        tracing::debug_span!("generate main traces", index).in_scope(|| {
            main_traces = records
                .par_iter()
                .map(|record| prover.generate_traces(record))
                .collect::<Vec<_>>();
        });

        let mut challenger = prover.config().challenger();
        pk.observe_into(&mut challenger);

        let proofs = records
            .into_par_iter()
            .zip(main_traces.into_par_iter())
            .map(|(record, main_trace)| {
                let main_data = prover.commit(&record, main_trace);
                prover
                    .open(pk, main_data, &mut challenger.clone())
                    .unwrap()
            })
            .collect::<Vec<_>>();

        Ok((proofs, report))
    }
}
