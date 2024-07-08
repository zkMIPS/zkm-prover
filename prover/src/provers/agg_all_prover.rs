use super::Prover;
use crate::contexts::AggAllContext;

use anyhow::Ok;

use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::plonk::config::PoseidonGoldilocksConfig;
use plonky2::plonk::proof::ProofWithPublicInputs;

use plonky2::util::timing::TimingTree;
use plonky2x::backend::circuit::Groth16WrapperParameters;
use plonky2x::backend::wrapper::wrap::WrappedCircuit;
use plonky2x::frontend::builder::CircuitBuilder as WrapperBuilder;
use std::time::Duration;

use plonky2::plonk::circuit_data::CircuitData;
use plonky2::util::serialization::{DefaultGateSerializer, DefaultGeneratorSerializer};
use std::marker::PhantomData;

use plonky2x::prelude::DefaultParameters;
use zkm_prover::proof::PublicValues;

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
        let proof_dir = ctx.proof_dir.clone();
        let pub_value_dir = ctx.pub_value_dir.clone();
        let output_dir = ctx.output_dir.clone();
        let _file = String::from("");
        let _args = "".to_string();

        if proof_num < 1 {
            return Ok(());
        }

        let mut timing = TimingTree::new("agg_all load from file", log::Level::Info);
        // read all proof and pub_value
        let mut root_proofs: Vec<ProofWithPublicInputs<F, C, D>> = Vec::new();
        let mut root_pub_values: Vec<PublicValues> = Vec::new();
        for seg_no in 0..proof_num {
            let proof_path = format!("{}/{}", proof_dir, seg_no);
            let root_proof_content = file::new(&proof_path).read_to_string()?;
            let root_proof: ProofWithPublicInputs<F, C, D> =
                serde_json::from_str(&root_proof_content)?;
            root_proofs.push(root_proof);

            let pub_value_path = format!("{}/{}", pub_value_dir, seg_no);
            let root_pub_value_content = file::new(&pub_value_path).read_to_string()?;
            let root_pub_value: PublicValues = serde_json::from_str(&root_pub_value_content)?;
            root_pub_values.push(root_pub_value);
        }

        timing.filter(Duration::from_millis(100)).print();
        timing = TimingTree::new("agg_all init all_circuits", log::Level::Info);
        let all_circuits = &*crate::provers::instance().lock().unwrap();
        timing.filter(Duration::from_millis(100)).print();

        let mut agg_proof: ProofWithPublicInputs<F, C, D> = root_proofs.first().unwrap().clone();
        let mut updated_agg_public_values: PublicValues = root_pub_values.first().unwrap().clone();

        let mut base_seg: usize = 1;
        let mut is_agg = false;

        timing = TimingTree::new("agg_all agg", log::Level::Info);
        if proof_num % 2 == 0 {
            let root_proof: ProofWithPublicInputs<F, C, D> = root_proofs.get(1).unwrap().clone();
            let public_values: PublicValues = root_pub_values.get(1).unwrap().clone();
            // Update public values for the aggregation.
            let agg_public_values = PublicValues {
                roots_before: updated_agg_public_values.roots_before,
                roots_after: public_values.roots_after,
                userdata: public_values.userdata,
            };
            // We can duplicate the proofs here because the state hasn't mutated.
            (agg_proof, updated_agg_public_values) = all_circuits.prove_aggregation(
                false,
                &agg_proof,
                false,
                &root_proof,
                agg_public_values.clone(),
            )?;
            all_circuits.verify_aggregation(&agg_proof)?;
            is_agg = true;
            base_seg = 2;
        }
        if proof_num > 2 {
            for i in 0..(proof_num - base_seg) / 2 {
                let index = base_seg + (i << 1);
                let root_proof_first: ProofWithPublicInputs<F, C, D> =
                    root_proofs.get(index).unwrap().clone();
                let first_public_values: PublicValues = root_pub_values.get(index).unwrap().clone();

                let index = base_seg + (i << 1) + 1;
                let root_proof: ProofWithPublicInputs<F, C, D> =
                    root_proofs.get(index).unwrap().clone();
                let public_values: PublicValues = root_pub_values.get(index).unwrap().clone();

                // Update public values for the aggregation.
                let new_agg_public_values = PublicValues {
                    roots_before: first_public_values.roots_before,
                    roots_after: public_values.roots_after,
                    userdata: public_values.userdata,
                };
                // We can duplicate the proofs here because the state hasn't mutated.
                let (new_agg_proof, new_updated_agg_public_values) = all_circuits
                    .prove_aggregation(
                        false,
                        &root_proof_first,
                        false,
                        &root_proof,
                        new_agg_public_values,
                    )?;

                // Update public values for the nested aggregation.
                let agg_public_values = PublicValues {
                    roots_before: updated_agg_public_values.roots_before,
                    roots_after: new_updated_agg_public_values.roots_after,
                    userdata: new_updated_agg_public_values.userdata,
                };

                // We can duplicate the proofs here because the state hasn't mutated.
                (agg_proof, updated_agg_public_values) = all_circuits.prove_aggregation(
                    is_agg,
                    &agg_proof,
                    true,
                    &new_agg_proof,
                    agg_public_values.clone(),
                )?;
                is_agg = true;
            }
        }
        timing.filter(Duration::from_millis(100)).print();

        timing = TimingTree::new("agg_all prove_block", log::Level::Info);
        let (block_proof, _block_public_values) =
            all_circuits.prove_block(None, &agg_proof, updated_agg_public_values)?;

        log::info!(
            "proof size: {:?}",
            serde_json::to_string(&block_proof.proof).unwrap().len()
        );
        let _result = all_circuits.verify_block(&block_proof);
        timing.filter(Duration::from_millis(100)).print();

        timing = TimingTree::new("agg_all circuit_data to_bytes", log::Level::Info);
        let gate_serializer = DefaultGateSerializer;
        let generator_serializer = DefaultGeneratorSerializer {
            _phantom: PhantomData::<C>,
        };
        let circuit_data = all_circuits
            .block
            .circuit
            .to_bytes(&gate_serializer, &generator_serializer)
            .unwrap();
        timing.filter(Duration::from_millis(100)).print();
        timing = TimingTree::new("agg_all circuit_data from_bytes", log::Level::Info);
        let circuit_data = CircuitData::<F, C, D>::from_bytes(
            circuit_data.as_slice(),
            &gate_serializer,
            &generator_serializer,
        )
        .unwrap();
        timing.filter(Duration::from_millis(100)).print();

        let path = output_dir.to_string();
        let builder = WrapperBuilder::<DefaultParameters, 2>::new();
        let mut circuit = builder.build();
        circuit.set_data(circuit_data);
        let wrapped_circuit = WrappedCircuit::<InnerParameters, OuterParameters, D>::build(circuit);

        timing = TimingTree::new("agg_all wrapped_circuit.prove", log::Level::Info);
        let wrapped_proof = wrapped_circuit.prove(&block_proof).unwrap();
        timing.filter(Duration::from_millis(100)).print();

        timing = TimingTree::new("agg_all write result", log::Level::Info);
        // save wrapper_proof
        file::new(&path).create_dir_all()?;
        let common_data_file = if path.ends_with('/') {
            format!("{}common_circuit_data.json", path)
        } else {
            format!("{}/common_circuit_data.json", path)
        };
        let _ =
            file::new(&common_data_file).write(&serde_json::to_vec(&wrapped_proof.common_data)?)?;
        let verify_data_file = if path.ends_with('/') {
            format!("{}verifier_only_circuit_data.json", path)
        } else {
            format!("{}/verifier_only_circuit_data.json", path)
        };
        let _ = file::new(&verify_data_file)
            .write(&serde_json::to_vec(&wrapped_proof.verifier_data)?)?;
        let proof_file = if path.ends_with('/') {
            format!("{}proof_with_public_inputs.json", path)
        } else {
            format!("{}/proof_with_public_inputs.json", path)
        };
        let _ = file::new(&proof_file).write(&serde_json::to_vec(&wrapped_proof.proof)?)?;
        timing.filter(Duration::from_millis(100)).print();

        Ok(())
    }
}
