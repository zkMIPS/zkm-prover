use super::Prover;
use crate::contexts::FinalContext;

use bincode::Error;
use elf::{endian::AnyEndian, ElfBytes};
use ethers::types::transaction::request;
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

        let proof_path = ctx.proof_path.clone();
        let pub_value_path = ctx.pub_value_path.clone();
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

        let root_proof_content = read_file_content(&proof_path)?;  
        let root_proof: ProofWithPublicInputs<F, C, D> = serde_json::from_str(&root_proof_content)?;

        let root_pub_value_content = read_file_content(&pub_value_path)?;
        let root_pub_value: PublicValues =  serde_json::from_str(&root_pub_value_content)?;

        let (block_proof, _block_public_values) =
        all_circuits.prove_block(None, &root_proof, root_pub_value)?;

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