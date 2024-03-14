use super::Prover;
use crate::contexts::FinalContext;

use bincode::Error;
use elf::{endian::AnyEndian, ElfBytes};
use num::ToPrimitive;
use serde::Serialize;
use uuid::timestamp::context;
use std::fs;
use std::time::Duration;

use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};
use plonky2::util::timing::TimingTree;
use plonky2::plonk::proof::ProofWithPublicInputs;

use plonky2x::backend::circuit::Groth16WrapperParameters;
use plonky2x::backend::wrapper::wrap::WrappedCircuit;
use plonky2x::frontend::builder::CircuitBuilder as WrapperBuilder;

use plonky2x::prelude::DefaultParameters;

use zkm::all_stark::AllStark;
use zkm::config::StarkConfig;
use zkm::cpu::kernel::assembler::segment_kernel;
use zkm::fixed_recursive_verifier::AllRecursiveCircuits;
use zkm::mips_emulator::state::{InstrumentedState, State, SEGMENT_STEPS};
use zkm::mips_emulator::utils::get_block_path;
use zkm::proof;
use zkm::proof::PublicValues;
use zkm::prover::prove;
use zkm::verifier::verify_proof;

use std::fs::File;  
use std::io::Write;
use std::io::Read;

use super::file_utils::read_file_content;

#[derive(Default)]
pub struct FinalProver {}

impl FinalProver {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Prover<FinalContext> for FinalProver {
    fn prove(&self, ctx: &FinalContext) -> anyhow::Result<()> {
        type InnerParameters = DefaultParameters;
        type OuterParameters = Groth16WrapperParameters;
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
        let output_dir = ctx.output_dir.clone();
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

        let (block_proof, _block_public_values) =
        all_circuits.prove_block(None, &agg_proof, updated_agg_public_values)?;

        log::info!(
            "proof size: {:?}",
            serde_json::to_string(&block_proof.proof).unwrap().len()
        );
        let result = all_circuits.verify_block(&block_proof);

        let path = format!("{}/", output_dir);
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