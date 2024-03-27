use super::Prover;
use crate::contexts::AggAllContext;

use anyhow::Ok;

use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::plonk::config::PoseidonGoldilocksConfig;
use plonky2::plonk::proof::ProofWithPublicInputs;

use plonky2x::backend::circuit::Groth16WrapperParameters;
use plonky2x::backend::wrapper::wrap::WrappedCircuit;
use plonky2x::frontend::builder::CircuitBuilder as WrapperBuilder;

use plonky2x::prelude::DefaultParameters;

use zkm::all_stark::AllStark;
use zkm::config::StarkConfig;
use zkm::fixed_recursive_verifier::AllRecursiveCircuits;
use zkm::proof::PublicValues;

use super::file_utils::read_file_content;

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

        let proof_num = ctx.proof_num as usize;
        let proof_dir = ctx.proof_dir.clone();
        let pub_value_dir = ctx.pub_value_dir.clone();
        let output_dir = ctx.output_dir.clone();
        let _file = String::from("");
        let _args = "".to_string();

        if proof_num < 1 {
            return Ok(());
        }

        // read all proof and pub_value
        let mut root_proofs: Vec<ProofWithPublicInputs<F, C, D>> = Vec::new();
        let mut root_pub_values: Vec<PublicValues> = Vec::new();

        for seg_no in 0..proof_num {
            let proof_path = format!("{}/{}", proof_dir, seg_no);
            let root_proof_content = read_file_content(&proof_path)?;
            let root_proof: ProofWithPublicInputs<F, C, D> =
                serde_json::from_str(&root_proof_content)?;
            root_proofs.push(root_proof);

            let pub_value_path = format!("{}/{}", pub_value_dir, seg_no);
            let root_pub_value_content = read_file_content(&pub_value_path)?;
            let root_pub_value: PublicValues = serde_json::from_str(&root_pub_value_content)?;
            root_pub_values.push(root_pub_value);
        }

        let all_stark = AllStark::<F, D>::default();
        let config = StarkConfig::standard_fast_config();
        // Preprocess all circuits.
        let all_circuits = AllRecursiveCircuits::<F, C, D>::new(
            &all_stark,
            // &[10..21, 15..22, 14..21, 9..21, 12..21, 15..23],
            &[10..21, 12..22, 13..21, 8..21, 10..21, 13..23],
            &config,
        );

        let mut agg_proof: ProofWithPublicInputs<F, C, D> = root_proofs.first().unwrap().clone();
        let mut updated_agg_public_values: PublicValues = root_pub_values.first().unwrap().clone();

        let mut base_seg: usize = 1;
        let mut is_agg = false;

        if proof_num % 2 == 0 {
            let root_proof: ProofWithPublicInputs<F, C, D> = root_proofs.get(1).unwrap().clone();
            let public_values: PublicValues = root_pub_values.get(1).unwrap().clone();
            // Update public values for the aggregation.
            let agg_public_values = PublicValues {
                roots_before: updated_agg_public_values.roots_before,
                roots_after: public_values.roots_after,
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

        let (block_proof, _block_public_values) =
            all_circuits.prove_block(None, &agg_proof, updated_agg_public_values)?;

        log::info!(
            "proof size: {:?}",
            serde_json::to_string(&block_proof.proof).unwrap().len()
        );
        let _result = all_circuits.verify_block(&block_proof);

        let path = output_dir.to_string();
        let builder = WrapperBuilder::<DefaultParameters, 2>::new();
        let mut circuit = builder.build();
        circuit.set_data(all_circuits.block.circuit);
        let wrapped_circuit = WrappedCircuit::<InnerParameters, OuterParameters, D>::build(circuit);
        println!("build finish");

        let wrapped_proof = wrapped_circuit.prove(&block_proof).unwrap();
        wrapped_proof.save(path).unwrap();
        Ok(())
    }
}
