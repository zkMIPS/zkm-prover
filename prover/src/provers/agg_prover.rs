use super::Prover;
use crate::contexts::AggContext;

use std::marker::PhantomData;

use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::plonk::config::PoseidonGoldilocksConfig;
use plonky2::util::timing::TimingTree;
use std::time::Duration;

use plonky2x::backend::circuit::Groth16WrapperParameters;
use plonky2x::backend::wrapper::wrap::WrappedCircuit;
use plonky2x::frontend::builder::CircuitBuilder as WrapperBuilder;
use plonky2x::prelude::DefaultParameters;

use plonky2::plonk::circuit_data::CircuitData;
use plonky2::util::serialization::{DefaultGateSerializer, DefaultGeneratorSerializer};

use zkm_prover::generation::state::Receipt;

use common::file;

#[derive(Default)]
pub struct AggProver {}

impl Prover<AggContext> for AggProver {
    fn prove(&self, ctx: &mut AggContext) -> anyhow::Result<()> {
        type InnerParameters = DefaultParameters;
        type OuterParameters = Groth16WrapperParameters;
        type F = GoldilocksField;
        const D: usize = 2;
        type C = PoseidonGoldilocksConfig;

        let receipt_path1 = ctx.receipt_path1.clone();
        let receipt_path2 = ctx.receipt_path2.clone();
        //let agg_receipt_path = ctx.agg_receipt_path.clone();
        let is_agg1 = ctx.is_agg_1;
        let is_agg2 = ctx.is_agg_2;
        let output_dir = ctx.output_dir.clone();

        let mut timing = TimingTree::new("agg init all_circuits", log::Level::Info);
        let all_circuits = &*crate::provers::instance().lock().unwrap();
        timing.filter(Duration::from_millis(100)).print();

        //let receipt_first_content = file::new(&receipt_path1).read_to_string()?;
        let receipt_first: Receipt<F, C, D> = serde_json::from_slice(&receipt_path1)?;

        //let receipt_next_content = file::new(&receipt_path2).read_to_string()?;
        let receipt_next: Receipt<F, C, D> = serde_json::from_slice(&receipt_path2)?;

        timing = TimingTree::new("agg agg", log::Level::Info);
        // We can duplicate the proofs here because the state hasn't mutated.
        let new_agg_receipt =
            all_circuits.prove_aggregation(is_agg1, &receipt_first, is_agg2, &receipt_next)?;
        timing.filter(Duration::from_millis(100)).print();
        all_circuits.verify_aggregation(&new_agg_receipt)?;

        // write receipt write file
        let json_string = serde_json::to_vec(&new_agg_receipt)?;
        //let _ = file::new(&agg_receipt_path).write(json_string.as_bytes())?;
        ctx.agg_receipt_path = json_string;

        if ctx.is_final {
            timing = TimingTree::new("agg prove_block", log::Level::Info);

            let block_receipt = all_circuits.prove_block(None, &new_agg_receipt)?;
            all_circuits.verify_block(&block_receipt)?;
            timing.filter(Duration::from_millis(100)).print();
            timing = TimingTree::new("agg circuit_data", log::Level::Info);
            let gate_serializer = DefaultGateSerializer;
            let generator_serializer = DefaultGeneratorSerializer {
                _phantom: PhantomData::<C>,
            };
            let circuit_data = all_circuits
                .block
                .circuit
                .to_bytes(&gate_serializer, &generator_serializer)
                .unwrap();
            let circuit_data = CircuitData::<F, C, D>::from_bytes(
                circuit_data.as_slice(),
                &gate_serializer,
                &generator_serializer,
            )
            .unwrap();

            let builder = WrapperBuilder::<DefaultParameters, 2>::new();
            let mut circuit = builder.build();
            circuit.set_data(circuit_data);
            let mut bit_size = vec![32usize; 16];
            bit_size.extend(vec![8; 32]);
            bit_size.extend(vec![64; 68]);
            let wrapped_circuit = WrappedCircuit::<InnerParameters, OuterParameters, D>::build(
                circuit,
                Some((vec![], bit_size)),
            );
            let wrapped_proof = wrapped_circuit.prove(&block_receipt.proof()).unwrap();
            wrapped_proof.save(output_dir.clone()).unwrap();

            let src_public_inputs = match &block_receipt {
                Receipt::Segments(receipt) => &receipt.proof.public_inputs,
                Receipt::Composite(recepit) => &recepit.program_receipt.proof.public_inputs,
            };

            let outdir_path = std::path::Path::new(&output_dir);

            let public_values_file = outdir_path.join("public_values.json");
            let _ = file::new(public_values_file.as_os_str().to_str().unwrap())
                .write(&serde_json::to_vec(&block_receipt.values())?)?;

            let block_public_inputs = serde_json::json!({
                "public_inputs": src_public_inputs,
            });
            let block_public_inputs_file = outdir_path.join("block_public_inputs.json");
            let _ = file::new(block_public_inputs_file.as_os_str().to_str().unwrap())
                .write(&serde_json::to_vec(&block_public_inputs)?)?;

            timing.filter(Duration::from_millis(100)).print();
        }

        Ok(())
    }
}
