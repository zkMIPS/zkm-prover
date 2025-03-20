use p3_field::PrimeField32;
use p3_maybe_rayon::prelude::*;
use std::io::Write;
use tracing_subscriber::fmt::format;
use zkm2_core_executor::{ExecutionState, Executor as Runtime, Program, ZKMContext};
use zkm2_core_machine::io::ZKMStdin;
use zkm2_core_machine::utils::ZKMCoreProverError;
use zkm2_core_machine::{CoreShapeConfig, MipsAir};
use zkm2_prover::ZKMProver;
use zkm2_prover::components::{DefaultProverComponents, ZKMProverComponents};
use zkm2_sdk::ProverClient;
use zkm2_stark::{
    Com, MachineProver, OpeningProof, PcsProverData, StarkGenericConfig, ZKMCoreOpts,
};

use crate::NetworkProve;
pub use crate::contexts::executor_context::SplitContext;
use crate::utils::get_block_path;
use common::file;

#[derive(Default)]
pub struct Executor {}
impl Executor {
    pub fn split(&self, ctx: &SplitContext) -> anyhow::Result<u64> {
        let elf_path = ctx.elf_path.clone();
        let block_no = ctx.block_no.unwrap_or(0);
        let seg_path = ctx.seg_path.clone();

        tracing::info!("split {} load elf file", elf_path);
        let elf = file::new(&elf_path).read()?;
        let block_path = get_block_path(&ctx.base_dir, &block_no.to_string(), "");
        let input_path = format!("{}input", block_path.trim_end_matches('/'));
        let input_data = file::new(&input_path).read()?;

        let mut network_prove = NetworkProve::new();
        network_prove.stdin.write(&input_data);

        let program = network_prove
            .prover
            .get_program(&elf)
            .map_err(|e| anyhow::Error::msg(e.to_string()))?;
        let (_, vk) = network_prove.prover.core_prover.setup(&program);
        let vk_bytes = bincode::serialize(&vk)?;
        file::new(&format!("{}/vk.bin", ctx.base_dir)).write(&vk_bytes)?;

        let context = network_prove.context_builder.build();
        let (segments, public_input_stream) = self.split_segments::<_, _>(
            &network_prove.prover.core_prover,
            program,
            &network_prove.stdin,
            network_prove.opts.core_opts,
            context,
            network_prove.prover.core_shape_config.as_ref(),
        )?;

        let total_steps = segments.last().map(|s| s.global_clk + 1).unwrap_or_default();

        file::new(&ctx.public_input_path).write(&public_input_stream)?;
        // Serialize and save each segment in parallel
        segments
            .par_iter()
            .enumerate()
            .try_for_each(|(index, segment)| -> anyhow::Result<()> {
                let segment_path = format!("{}/{}", seg_path, index);
                let encoded_segment = bincode::serialize(segment)?;
                file::new(&segment_path).write(&encoded_segment)?;
                Ok(())
            })?;

        Ok(total_steps)
    }

    /// Splits the execution of a given program into multiple execution state segments.
    ///
    /// This function initializes an execution runtime with the provided program and execution context.
    /// It processes input data, including standard input and any provided proofs, then iterates through
    /// execution states, collecting checkpoints along the way.
    ///
    /// # Parameters
    /// - `program`: The program to be executed.
    /// - `stdin`: Input data buffer containing program input and proofs.
    /// - `opts`: Execution options.
    /// - `context`: The execution context.
    /// - `shape_config`: Optional configuration for shape constraints.
    ///
    /// # Returns
    /// - `Ok((Vec<ExecutionState>, Vec<u8>))`: A tuple containing a vector of execution state checkpoints
    ///   and the public values stream from the execution.
    /// - `Err(ZKMCoreProverError)`: If execution encounters an error.
    fn split_segments<SC: StarkGenericConfig, P: MachineProver<SC, MipsAir<SC::Val>>>(
        &self,
        _prover: &P,
        program: Program,
        stdin: &ZKMStdin,
        opts: ZKMCoreOpts,
        context: ZKMContext,
        shape_config: Option<&CoreShapeConfig<SC::Val>>,
    ) -> Result<(Vec<ExecutionState>, Vec<u8>), ZKMCoreProverError>
    where
        SC: StarkGenericConfig,
        P: MachineProver<SC, MipsAir<SC::Val>>,
        SC::Val: PrimeField32,
        SC::Challenger: 'static + Clone + Send,
        OpeningProof<SC>: Send,
        Com<SC>: Send + Sync,
        PcsProverData<SC>: Send + Sync,
    {
        let mut runtime = Runtime::with_context(program.clone(), opts, context);
        runtime.maximal_shapes = shape_config.map(|config| {
            config
                .maximal_core_shapes()
                .into_iter()
                .map(|s| s.inner)
                .collect()
        });
        runtime.write_vecs(&stdin.buffer);

        for (proof, vk) in stdin.proofs.iter().cloned() {
            runtime.write_proof(proof, vk);
        }

        let mut checkpoints = vec![];
        while let Ok((checkpoint, done)) = runtime
            .execute_state(false)
            .map_err(ZKMCoreProverError::ExecutionError)
        {
            checkpoints.push(checkpoint);
            if done {
                break;
            }
        }

        Ok((checkpoints, runtime.state.public_values_stream))
    }
}

#[test]
fn test_split_segments() {
    println!("0");
}
