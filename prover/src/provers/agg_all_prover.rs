use super::Prover;
use crate::contexts::AggAllContext;

use anyhow::Ok;

use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::plonk::config::PoseidonGoldilocksConfig;

use plonky2::util::timing::TimingTree;
use plonky2x::backend::circuit::Groth16WrapperParameters;
use plonky2x::backend::wrapper::wrap::WrappedCircuit;
use plonky2x::frontend::builder::CircuitBuilder as WrapperBuilder;
use std::time::Duration;
use zkm_prover::generation::state::Receipt;

use plonky2::plonk::circuit_data::CircuitData;
use plonky2::util::serialization::{DefaultGateSerializer, DefaultGeneratorSerializer};
use std::marker::PhantomData;

use plonky2x::prelude::DefaultParameters;

use common::file;

#[derive(Default)]
pub struct AggAllProver {}

impl AggAllProver {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Prover<AggAllContext> for AggAllProver {
    fn prove(&self, ctx: &AggAllContext) -> anyhow::Result<()> {
        type InnerParameters = DefaultParameters;
        type OuterParameters = Groth16WrapperParameters;
        type F = GoldilocksField;
        const D: usize = 2;
        type C = PoseidonGoldilocksConfig;

        // let seg_size = ctx.seg_size as usize;
        let proof_num = ctx.proof_num as usize;
        let receipt_dir = ctx.receipt_dir.clone();
        let output_dir = ctx.output_dir.clone();
        let _file = String::from("");
        let _args = "".to_string();

        if proof_num < 1 {
            return Ok(());
        }

        let mut timing = TimingTree::new("agg_all load from file", log::Level::Info);
        let mut receipts: Vec<Receipt<F, C, D>> = Vec::new();
        for seg_no in 0..proof_num {
            let recepit_path = format!("{}/{}", receipt_dir, seg_no);
            let receipt_content = file::new(&recepit_path).read_to_string()?;
            let receipt: Receipt<F, C, D> = serde_json::from_str(&receipt_content)?;
            receipts.push(receipt);
        }

        timing.filter(Duration::from_millis(100)).print();
        timing = TimingTree::new("agg_all init all_circuits", log::Level::Info);
        let all_circuits = &*crate::provers::instance().lock().unwrap();
        timing.filter(Duration::from_millis(100)).print();

        let mut agg_receipt: Receipt<F, C, D> = receipts.first().unwrap().clone();
        let mut base_seg: usize = 1;
        let mut is_agg = false;

        timing = TimingTree::new("agg_all agg", log::Level::Info);
        if proof_num % 2 == 0 {
            let receipt: Receipt<F, C, D> = receipts.get(1).unwrap().clone();
            agg_receipt = all_circuits.prove_aggregation(false, &agg_receipt, false, &receipt)?;
            timing.filter(Duration::from_millis(100)).print();
            all_circuits.verify_aggregation(&agg_receipt)?;
            is_agg = true;
            base_seg = 2;
        }
        if proof_num > 2 {
            for i in 0..(proof_num - base_seg) / 2 {
                let index = base_seg + (i << 1);
                let first_receipt: Receipt<F, C, D> = receipts.get(index).unwrap().clone();

                let index = base_seg + (i << 1) + 1;
                let receipt: Receipt<F, C, D> = receipts.get(index).unwrap().clone();

                timing = TimingTree::new("prove aggression", log::Level::Info);
                let new_agg_receipt =
                    all_circuits.prove_aggregation(false, &first_receipt, false, &receipt)?;
                timing.filter(Duration::from_millis(100)).print();
                all_circuits.verify_aggregation(&new_agg_receipt)?;

                timing = TimingTree::new("prove nested aggression", log::Level::Info);

                agg_receipt =
                    all_circuits.prove_aggregation(is_agg, &agg_receipt, true, &new_agg_receipt)?;
                timing.filter(Duration::from_millis(100)).print();
                all_circuits.verify_aggregation(&agg_receipt)?;
                is_agg = true;
            }
        }
        timing.filter(Duration::from_millis(100)).print();

        timing = TimingTree::new("agg_all prove_block", log::Level::Info);
        let block_receipt = all_circuits.prove_block(None, &agg_receipt)?;
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

        Ok(())
    }
}
