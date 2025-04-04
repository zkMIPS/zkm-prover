use crate::contexts::AggContext;
use crate::NetworkProve;
use zkm2_core_executor::ZKMReduceProof;
use zkm2_prover::build::Witnessable;
use zkm2_prover::{
    CoreSC, InnerSC, ZKMCircuitWitness, ZKMProver, ZKMRecursionProverError, ZKMVerifyingKey,
};
use zkm2_recursion_circuit::machine::{ZKMCompressWitnessValues, ZKMRecursionWitnessValues};
use zkm2_recursion_compiler::config::InnerConfig;
use zkm2_recursion_core::Runtime;
use zkm2_stark::{
    Challenge, MachineProver, ShardProof, StarkGenericConfig, Val, ZKMCoreOpts, ZKMProverOpts,
};

#[derive(Default)]
pub struct AggProver {}

impl AggProver {
    pub fn prove(&self, ctx: &AggContext) -> anyhow::Result<Vec<u8>> {
        let network_prove = NetworkProve::new();
        let input = if ctx.is_leaf_layer {
            let shard_proofs = ctx
                .proofs
                .iter()
                .map(|proof| bincode::deserialize(proof).unwrap())
                .collect();
            let vk = bincode::deserialize(&ctx.vk)?;
            ZKMCircuitWitness::Core(ZKMRecursionWitnessValues {
                vk,
                shard_proofs,
                is_complete: ctx.is_complete,
                is_first_shard: ctx.is_first_shard,
                vk_root: network_prove.prover.recursion_vk_root,
            })
        } else {
            let reduced_proofs: Vec<ZKMReduceProof<_>> = ctx
                .proofs
                .iter()
                .map(|vk_and_proof| bincode::deserialize(vk_and_proof).unwrap())
                .collect();

            ZKMCircuitWitness::Compress(ZKMCompressWitnessValues {
                vks_and_proofs: reduced_proofs
                    .into_iter()
                    .map(|proof| (proof.vk, proof.proof))
                    .collect(),
                is_complete: ctx.is_complete,
            })
        };

        let reduced_proof = self.compress(
            &network_prove.prover,
            input,
            network_prove.opts.recursion_opts,
        )?;

        Ok(bincode::serialize(&reduced_proof)?)
    }

    fn compress(
        &self,
        prover: &ZKMProver,
        input: ZKMCircuitWitness,
        recursion_opts: ZKMCoreOpts,
    ) -> anyhow::Result<ZKMReduceProof<InnerSC>> {
        // Get the program and witness stream.
        let (program, witness_stream) = tracing::debug_span!("get program and witness stream")
            .in_scope(|| match input {
                ZKMCircuitWitness::Core(input) => {
                    let mut witness_stream = Vec::new();
                    Witnessable::<InnerConfig>::write(&input, &mut witness_stream);
                    (prover.recursion_program(&input), witness_stream)
                }
                ZKMCircuitWitness::Deferred(input) => {
                    let mut witness_stream = Vec::new();
                    Witnessable::<InnerConfig>::write(&input, &mut witness_stream);
                    (prover.deferred_program(&input), witness_stream)
                }
                ZKMCircuitWitness::Compress(input) => {
                    let mut witness_stream = Vec::new();

                    let input_with_merkle = prover.make_merkle_proofs(input);

                    Witnessable::<InnerConfig>::write(&input_with_merkle, &mut witness_stream);

                    (prover.compress_program(&input_with_merkle), witness_stream)
                }
            });

        // Execute the runtime.
        let record = tracing::debug_span!("execute runtime").in_scope(|| {
            let mut runtime = Runtime::<Val<InnerSC>, Challenge<InnerSC>, _>::new(
                program.clone(),
                prover.compress_prover.config().perm.clone(),
            );
            runtime.witness_stream = witness_stream.into();
            runtime
                .run()
                .map_err(|e| ZKMRecursionProverError::RuntimeError(e.to_string()))
                .unwrap();
            runtime.record
        });

        // Generate the dependencies.
        let mut records = vec![record];
        tracing::debug_span!("generate dependencies").in_scope(|| {
            prover.compress_prover.machine().generate_dependencies(
                &mut records,
                &recursion_opts,
                None,
            )
        });

        // Generate the traces.
        let record = records.into_iter().next().unwrap();
        let traces = tracing::debug_span!("generate traces")
            .in_scope(|| prover.compress_prover.generate_traces(&record));

        let (vk, proof) = tracing::debug_span!("batch").in_scope(|| {
            // Get the keys.
            let (pk, vk) = tracing::debug_span!("Setup compress program")
                .in_scope(|| prover.compress_prover.setup(&program));

            // Observe the proving key.
            let mut challenger = prover.compress_prover.config().challenger();
            tracing::debug_span!("observe proving key").in_scope(|| {
                pk.observe_into(&mut challenger);
            });

            // Commit to the record and traces.
            let data = tracing::debug_span!("commit")
                .in_scope(|| prover.compress_prover.commit(&record, traces));

            // Generate the proof.
            let proof = tracing::debug_span!("open").in_scope(|| {
                prover
                    .compress_prover
                    .open(&pk, data, &mut challenger)
                    .unwrap()
            });

            // Verify the proof.
            #[cfg(feature = "debug")]
            prover
                .compress_prover
                .machine()
                .verify(
                    &vk,
                    &zkm2_stark::MachineProof {
                        shard_proofs: vec![proof.clone()],
                    },
                    &mut prover.compress_prover.config().challenger(),
                )
                .unwrap();

            (vk, proof)
        });

        Ok(ZKMReduceProof { vk, proof })
    }
}
