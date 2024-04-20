use super::Prover;
use crate::contexts::AggContext;
use crate::provers::select_degree_bits;

use num::ToPrimitive;
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

use common::file;

#[derive(Default)]
pub struct AggProver {}

impl AggProver {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Prover<AggContext> for AggProver {
    fn prove(&self, ctx: &AggContext) -> anyhow::Result<()> {
        type InnerParameters = DefaultParameters;
        type OuterParameters = Groth16WrapperParameters;
        type F = GoldilocksField;
        const D: usize = 2;
        type C = PoseidonGoldilocksConfig;

        let _basedir = ctx.basedir.clone();
        let _block_no = ctx.block_no.to_string();
        let seg_size = ctx.seg_size.to_usize().expect("u32->usize failed");
        let proof_path1 = ctx.proof_path1.clone();
        let proof_path2 = ctx.proof_path2.clone();
        let pub_value_path1 = ctx.pub_value_path1.clone();
        let pub_value_path2 = ctx.pub_value_path2.clone();
        let agg_proof_path = ctx.agg_proof_path.clone();
        let agg_pub_value_path = ctx.agg_pub_value_path.clone();
        let is_agg1 = ctx.is_agg_1;
        let is_agg2 = ctx.is_agg_2;
        let output_dir = ctx.output_dir.clone();
        let _file = String::from("");
        let _args = "".to_string();

        let all_stark = AllStark::<F, D>::default();
        let config = StarkConfig::standard_fast_config();
        // Preprocess all circuits.
        let all_circuits = AllRecursiveCircuits::<F, C, D>::new(
            &all_stark,
            &select_degree_bits(seg_size),
            &config,
        );

        let root_proof_content = file::new(&proof_path1).read_to_string()?;
        let root_proof: ProofWithPublicInputs<F, C, D> = serde_json::from_str(&root_proof_content)?;

        let next_proof_content = file::new(&proof_path2).read_to_string()?;
        let next_proof: ProofWithPublicInputs<F, C, D> = serde_json::from_str(&next_proof_content)?;

        let root_pub_value_content = file::new(&pub_value_path1).read_to_string()?;
        let root_pub_value: PublicValues = serde_json::from_str(&root_pub_value_content)?;

        let next_pub_value_content = file::new(&pub_value_path2).read_to_string()?;
        let next_pub_value: PublicValues = serde_json::from_str(&next_pub_value_content)?;

        // Update public values for the aggregation.
        let agg_public_values = PublicValues {
            roots_before: root_pub_value.roots_before,
            roots_after: next_pub_value.roots_after,
        };
        // We can duplicate the proofs here because the state hasn't mutated.
        let (agg_proof, updated_agg_public_values) = all_circuits.prove_aggregation(
            is_agg1,
            &root_proof,
            is_agg2,
            &next_proof,
            agg_public_values.clone(),
        )?;
        all_circuits.verify_aggregation(&agg_proof)?;

        // write agg_proof write file
        let json_string = serde_json::to_string(&agg_proof)?;
        let _ = file::new(&agg_proof_path).write(json_string.as_bytes())?;

        // updated_agg_public_values file
        let json_string = serde_json::to_string(&updated_agg_public_values)?;
        let _ = file::new(&agg_pub_value_path).write(json_string.as_bytes())?;

        if ctx.is_final {
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
            let wrapped_circuit =
                WrappedCircuit::<InnerParameters, OuterParameters, D>::build(circuit);
            println!("build finish");

            let wrapped_proof = wrapped_circuit.prove(&block_proof).unwrap();
            // save wrapper_proof
            file::new(&path).create_dir_all()?;
            let common_data_file = if path.ends_with('/') {
                format!("{}common_circuit_data.json", path)
            } else {
                format!("{}/common_circuit_data.json", path)
            };
            let _ = file::new(&common_data_file)
                .write(&serde_json::to_vec(&wrapped_proof.common_data)?)?;
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
        }

        Ok(())
    }
}
