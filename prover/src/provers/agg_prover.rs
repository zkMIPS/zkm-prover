use super::Prover;
use crate::contexts::AggContext;

use std::time::Duration;
use num::ToPrimitive;
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};
use plonky2::util::timing::TimingTree;
use plonky2::plonk::proof::ProofWithPublicInputs;

// use plonky2x::backend::wrapper::wrap::WrappedCircuit;
// use plonky2x::frontend::builder::CircuitBuilder as WrapperBuilder;


use mips_circuits::all_stark::AllStark;
use mips_circuits::config::StarkConfig;
use mips_circuits::cpu::kernel::assembler::segment_kernel;
use mips_circuits::fixed_recursive_verifier::AllRecursiveCircuits;
use mips_circuits::mips_emulator::state::{InstrumentedState, State, SEGMENT_STEPS};
use mips_circuits::mips_emulator::utils::get_block_path;
use mips_circuits::proof;
use mips_circuits::proof::PublicValues;
use mips_circuits::prover::prove;
use mips_circuits::verifier::verify_proof;

use std::fs::File;  
use std::io::Write;
use std::io::Read;

use super::file_utils::read_file_content;

#[derive(Default)]
pub struct AggProver {}

impl AggProver {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Prover<AggContext> for AggProver {
    fn prove(&self, ctx: &AggContext) -> anyhow::Result<()> {
        type F = GoldilocksField;
        const D: usize = 2;
        type C = PoseidonGoldilocksConfig;

        let basedir = ctx.basedir.clone();
        let block_no = ctx.block_no.to_string();
        let seg_size = ctx.seg_size.to_usize().expect("u32->usize failed");
        let proof_path1 = ctx.proof_path1.clone();
        let proof_path2 = ctx.proof_path2.clone();
        let pub_value_path1 = ctx.pub_value_path1.clone();
        let pub_value_path2 = ctx.pub_value_path2.clone();
        let agg_proof_path = ctx.agg_proof_path.clone();
        let agg_pub_value_path = ctx.agg_pub_value_path.clone();
        let file = String::from("");
        let args = "".to_string();


        let all_stark = AllStark::<F, D>::default();
        let config = StarkConfig::standard_fast_config();
        // Preprocess all circuits.
        let all_circuits = AllRecursiveCircuits::<F, C, D>::new(
            &all_stark,
            &[10..21, 15..22, 14..21, 9..21, 12..21, 15..23],
            &config,
        );

        let root_proof_content = read_file_content(&proof_path1)?;  
        let root_proof: ProofWithPublicInputs<F, C, D> = serde_json::from_str(&root_proof_content)?;

        let next_proof_content = read_file_content(&proof_path2)?;  
        let next_proof: ProofWithPublicInputs<F, C, D> = serde_json::from_str(&next_proof_content)?;

        let root_pub_value_content = read_file_content(&pub_value_path1)?;
        let root_pub_value: PublicValues =  serde_json::from_str(&root_pub_value_content)?;

        let next_pub_value_content = read_file_content(&pub_value_path2)?;
        let next_pub_value: PublicValues =  serde_json::from_str(&next_pub_value_content)?;

        let mut timing = TimingTree::new("agg first", log::Level::Info);
        // Update public values for the aggregation.
        let agg_public_values = PublicValues {
            roots_before: root_pub_value.roots_before,
            roots_after: next_pub_value.roots_after,
        };
        timing = TimingTree::new("prove aggression", log::Level::Info);
        // We can duplicate the proofs here because the state hasn't mutated.
        let (agg_proof, updated_agg_public_values) = all_circuits.prove_aggregation(
            false,
            &next_proof,
            false,
            &root_proof,
            agg_public_values.clone(),
        )?;
        timing.filter(Duration::from_millis(100)).print();
        all_circuits.verify_aggregation(&agg_proof)?;

        // write agg_proof write file
        let json_string = serde_json::to_string(&agg_proof)?;  
        let mut file = File::create(agg_proof_path)?;
        file.write_all(json_string.as_bytes())?;
        file.flush()?;

        // updated_agg_public_values file
        let json_string = serde_json::to_string(&updated_agg_public_values)?;  
        let mut file = File::create(agg_pub_value_path)?;
        file.write_all(json_string.as_bytes())?;
        file.flush()?;

        Ok(())
    }
}