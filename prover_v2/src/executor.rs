use common::file;
use std::fs::File;
use std::io::{self, Seek, Write};
use std::sync::{
    mpsc::sync_channel,
    {Arc, Mutex},
};
use std::thread::ScopedJoinHandle;
use std::time::Instant;

use p3_field::PrimeField32;
use p3_matrix::dense::RowMajorMatrix;
use p3_maybe_rayon::prelude::*;
use tracing_subscriber::fmt::format;
use zkm2_core_executor::{
    events::{format_table_line, sorted_table_lines},
    ExecutionRecord, ExecutionReport, ExecutionState, Executor as Runtime, Program, ZKMContext,
};
use zkm2_core_machine::{
    io::ZKMStdin,
    utils::{concurrency::TurnBasedSync, trace_checkpoint, ZKMCoreProverError},
    CoreShapeConfig, CostEstimator, MipsAir,
};
use zkm2_prover::components::{DefaultProverComponents, ZKMProverComponents};
use zkm2_prover::ZKMProver;
use zkm2_sdk::ProverClient;
use zkm2_stark::{
    Com, MachineProver, MachineProvingKey, MachineRecord, OpeningProof, PcsProverData,
    PublicValues, StarkGenericConfig, Val, ZKMCoreOpts,
};

pub use crate::contexts::SplitContext;
use crate::utils::get_block_path;
use crate::NetworkProve;

#[derive(Default)]
pub struct Executor {}
impl Executor {
    pub fn split(&self, ctx: &SplitContext) -> anyhow::Result<u64> {
        let elf_path = ctx.elf_path.clone();
        // let block_no = ctx.block_no.unwrap_or(0);
        // let seg_path = ctx.seg_path.clone();

        tracing::info!("split {} load elf file", elf_path);
        let elf = file::new(&elf_path).read()?;
        // let block_path = get_block_path(&ctx.base_dir, &block_no.to_string(), "");
        // let input_path = format!("{}/input", block_path.trim_end_matches('/'));
        let input_data = file::new(&ctx.private_input_path).read()?;

        let mut network_prove = NetworkProve::new_with_segment_size(ctx.seg_size);
        // TODO: add more input
        network_prove.stdin.write(&input_data);

        let program = network_prove
            .prover
            .get_program(&elf)
            .map_err(|e| anyhow::Error::msg(e.to_string()))?;
        let (_, vk) = network_prove.prover.core_prover.setup(&program);
        let vk_bytes = bincode::serialize(&vk)?;
        file::new(&format!("{}/vk.bin", ctx.base_dir)).write(&vk_bytes)?;

        let context = network_prove.context_builder.build();
        let (total_steps, public_values_stream) = self.split_with_context::<_, _>(
            &network_prove.prover.core_prover,
            ctx,
            program,
            &network_prove.stdin,
            network_prove.opts.core_opts,
            context,
            network_prove.prover.core_shape_config.as_ref(),
        )?;
        // write public_values_stream to output_path
        file::new(&ctx.output_path).write(&public_values_stream)?;

        Ok(total_steps)
    }

    // _prover is used for type inference.
    pub fn split_with_context<SC: StarkGenericConfig, P: MachineProver<SC, MipsAir<SC::Val>>>(
        &self,
        _prover: &P,
        ctx: &SplitContext,
        program: Program,
        stdin: &ZKMStdin,
        opts: ZKMCoreOpts,
        context: ZKMContext,
        shape_config: Option<&CoreShapeConfig<SC::Val>>,
    ) -> anyhow::Result<(u64, Vec<u8>)>
    where
        SC::Val: PrimeField32,
        SC::Challenger: 'static + Clone + Send,
        OpeningProof<SC>: Send,
        Com<SC>: Send + Sync,
        PcsProverData<SC>: Send + Sync,
    {
        // Setup the runtime.
        let mut runtime = Runtime::with_context(program.clone(), opts, context);
        runtime.maximal_shapes = shape_config.map(|config| {
            config
                .maximal_core_shapes()
                .into_iter()
                .map(|s| s.inner)
                .collect()
        });
        runtime.write_vecs(&stdin.buffer);
        for proof in stdin.proofs.iter() {
            let (proof, vk) = proof.clone();
            runtime.write_proof(proof, vk);
        }

        // Record the start of the process.
        let split_start = Instant::now();
        let span = tracing::Span::current().clone();
        std::thread::scope(move |s| {
            let _span = span.enter();

            // Spawn the checkpoint generator thread.
            let checkpoint_generator_span = tracing::Span::current().clone();
            let (checkpoints_tx, checkpoints_rx) =
                sync_channel::<(usize, File, bool)>(opts.checkpoints_channel_capacity);
            let checkpoint_generator_handle: ScopedJoinHandle<Result<_, ZKMCoreProverError>> = s
                .spawn(move || {
                    let _span = checkpoint_generator_span.enter();
                    tracing::debug_span!("checkpoint generator").in_scope(|| {
                        let mut index = 0;

                        loop {
                            // Enter the span.
                            let span = tracing::debug_span!("batch");
                            let _span = span.enter();

                            // Execute the runtime until we reach a checkpoint.t
                            let (checkpoint, done) = runtime
                                .execute_state(false)
                                .map_err(ZKMCoreProverError::ExecutionError)?;

                            // Save the checkpoint to a temp file.
                            let mut checkpoint_file =
                                tempfile::tempfile().map_err(ZKMCoreProverError::IoError)?;
                            checkpoint
                                .save(&mut checkpoint_file)
                                .map_err(ZKMCoreProverError::IoError)?;

                            // Send the checkpoint.
                            checkpoints_tx.send((index, checkpoint_file, done)).unwrap();

                            // If we've reached the final checkpoint, break out of the loop.
                            if done {
                                break Ok(runtime.state.public_values_stream);
                            }

                            // Update the index.
                            index += 1;
                        }
                    })
                });

            // Spawn the phase 2 record generator thread.
            let p2_record_gen_sync = Arc::new(TurnBasedSync::new());
            let checkpoints_rx = Arc::new(Mutex::new(checkpoints_rx));
            let segment_index = Arc::new(Mutex::new(0));

            let report_aggregate = Arc::new(Mutex::new(ExecutionReport::default()));
            let state = Arc::new(Mutex::new(PublicValues::<u32, u32>::default().reset()));
            let deferred = Arc::new(Mutex::new(ExecutionRecord::new(program.clone().into())));
            let mut p2_record_and_trace_gen_handles = Vec::new();
            for _ in 0..opts.trace_gen_workers {
                let record_gen_sync = Arc::clone(&p2_record_gen_sync);
                let checkpoints_rx = Arc::clone(&checkpoints_rx);
                let segment_index = Arc::clone(&segment_index);

                let report_aggregate = Arc::clone(&report_aggregate);
                let state = Arc::clone(&state);
                let deferred = Arc::clone(&deferred);
                let program = program.clone();

                let span = tracing::Span::current().clone();

                let handle = s.spawn(move || {
                    let _span = span.enter();
                    tracing::debug_span!("phase 2 trace generation").in_scope(|| {
                        loop {
                            // Receive the latest checkpoint.
                            let received = { checkpoints_rx.lock().unwrap().recv() };
                            if let Ok((index, mut checkpoint, done)) = received {
                                // Trace the checkpoint and reconstruct the execution records.
                                let (mut records, report) =
                                    tracing::debug_span!("trace checkpoint").in_scope(|| {
                                        let mut reader = io::BufReader::new(&checkpoint);
                                        let state: ExecutionState =
                                            bincode::deserialize_from(&mut reader)
                                                .expect("failed to deserialize state");
                                        trace_checkpoint::<SC>(
                                            program.clone(),
                                            state,
                                            opts,
                                            shape_config,
                                        )
                                    });
                                tracing::info!("generated {} records", records.len());
                                *report_aggregate.lock().unwrap() += report;
                                // reset_seek(&mut checkpoint);
                                checkpoint
                                    .seek(io::SeekFrom::Start(0))
                                    .expect("failed to seek to start of tempfile");

                                // Wait for our turn to update the state.
                                record_gen_sync.wait_for_turn(index);

                                // Update the public values & prover state for the shards which contain
                                // "cpu events".
                                let mut state = state.lock().unwrap();
                                for record in records.iter_mut() {
                                    state.shard += 1;
                                    state.execution_shard = record.public_values.execution_shard;
                                    state.start_pc = record.public_values.start_pc;
                                    state.next_pc = record.public_values.next_pc;
                                    state.committed_value_digest =
                                        record.public_values.committed_value_digest;
                                    state.deferred_proofs_digest =
                                        record.public_values.deferred_proofs_digest;
                                    record.public_values = *state;
                                }

                                // Defer events that are too expensive to include in every shard.
                                let mut deferred = deferred.lock().unwrap();
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
                                    state.previous_init_addr_bits =
                                        record.public_values.previous_init_addr_bits;
                                    state.last_init_addr_bits =
                                        record.public_values.last_init_addr_bits;
                                    state.previous_finalize_addr_bits =
                                        record.public_values.previous_finalize_addr_bits;
                                    state.last_finalize_addr_bits =
                                        record.public_values.last_finalize_addr_bits;
                                    state.start_pc = state.next_pc;
                                    record.public_values = *state;
                                }
                                records.append(&mut deferred);

                                let mut segment_index = segment_index.lock().unwrap();
                                let base_index = *segment_index;
                                *segment_index += records.len();

                                // // Generate the dependencies.
                                // tracing::debug_span!("generate dependencies", index).in_scope(|| {
                                //     prover.machine().generate_dependencies(&mut records, &opts, None);
                                // });

                                // Let another worker update the state.
                                record_gen_sync.advance_turn();

                                records.par_iter().enumerate().for_each(|(i, record)| {
                                    let encoded_record = bincode::serialize(&record).unwrap();
                                    file::new(&format!("{}/{}", ctx.seg_path, base_index + i))
                                        .write(&encoded_record)
                                        .expect("Failed to write record");
                                });

                                // // Fix the shape of the records.
                                // if let Some(shape_config) = shape_config {
                                //     for record in records.iter_mut() {
                                //         shape_config.fix_shape(record).unwrap();
                                //     }
                                // }

                                // trace_gen_sync.wait_for_turn(index);
                                //
                                // // Send the records to the phase 2 prover.
                                // let chunked_records = chunk_vec(records, opts.shard_batch_size);
                                // let chunked_main_traces = chunk_vec(main_traces, opts.shard_batch_size);
                                // chunked_records
                                //     .into_iter()
                                //     .zip(chunked_main_traces.into_iter())
                                //     .for_each(|(records, main_traces)| {
                                //         records_and_traces_tx
                                //             .lock()
                                //             .unwrap()
                                //             .send((records, main_traces))
                                //             .unwrap();
                                //     });
                                //
                                // trace_gen_sync.advance_turn();
                            } else {
                                break;
                            }
                        }
                    })
                });
                p2_record_and_trace_gen_handles.push(handle);
            }
            // Wait until the checkpoint generator handle has fully finished.
            let public_values_stream = checkpoint_generator_handle.join().unwrap().unwrap();
            // file::new(&ctx.public_input_path).write(&public_values_stream)?;                    // write public_values_stream

            // Wait until the records and traces have been fully generated for phase 2.
            p2_record_and_trace_gen_handles
                .into_iter()
                .for_each(|handle| handle.join().unwrap());

            // Log some of the `ExecutionReport` information.
            let report_aggregate = report_aggregate.lock().unwrap();
            tracing::info!(
                "execution report (totals): total_cycles={}, total_syscall_cycles={}, touched_memory_addresses={}",
                report_aggregate.total_instruction_count(),
                report_aggregate.total_syscall_count(),
                report_aggregate.touched_memory_addresses,
            );

            // Print the opcode and syscall count tables like `du`: sorted by count (descending) and
            // with the count in the first column.
            tracing::info!("execution report (opcode counts):");
            let (width, lines) = sorted_table_lines(report_aggregate.opcode_counts.as_ref());
            for (label, count) in lines {
                if *count > 0 {
                    tracing::info!("  {}", format_table_line(&width, &label, count));
                } else {
                    tracing::debug!("  {}", format_table_line(&width, &label, count));
                }
            }

            tracing::info!("execution report (syscall counts):");
            let (width, lines) = sorted_table_lines(report_aggregate.syscall_counts.as_ref());
            for (label, count) in lines {
                if *count > 0 {
                    tracing::info!("  {}", format_table_line(&width, &label, count));
                } else {
                    tracing::debug!("  {}", format_table_line(&width, &label, count));
                }
            }

            let cycles = report_aggregate.total_instruction_count();

            // Print the summary.
            let split_time = split_start.elapsed().as_secs_f64();
            tracing::info!(
                "summary: cycles={}, gas={}, executor={}s, khz={:.2}",
                cycles,
                report_aggregate.estimate_gas(),
                split_time,
                (cycles as f64 / (split_time * 1000.0) as f64),
            );

            Ok((cycles, public_values_stream))
        })
    }
}

#[test]
fn test_split_segments() {
    println!("0");
}
