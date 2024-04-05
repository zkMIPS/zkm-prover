use super::Prover;
use crate::contexts::AggContext;
use crate::provers::select_degree_bits;

use num::ToPrimitive;
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::plonk::config::PoseidonGoldilocksConfig;
use plonky2::plonk::proof::ProofWithPublicInputs;

use zkm::all_stark::AllStark;
use zkm::config::StarkConfig;
use zkm::fixed_recursive_verifier::AllRecursiveCircuits;
use zkm::proof::PublicValues;

use common::file::{read_to_string, write_file};

#[derive(Default)]
pub struct AggProver {}

impl AggProver {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Prover<AggContext> for AggProver {
    async fn prove(&self, ctx: &AggContext) -> anyhow::Result<()> {
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

        let root_proof_content = read_to_string(&proof_path1).await?;
        let root_proof: ProofWithPublicInputs<F, C, D> = serde_json::from_str(&root_proof_content)?;

        let next_proof_content = read_to_string(&proof_path2).await?;
        let next_proof: ProofWithPublicInputs<F, C, D> = serde_json::from_str(&next_proof_content)?;

        let root_pub_value_content = read_to_string(&pub_value_path1).await?;
        let root_pub_value: PublicValues = serde_json::from_str(&root_pub_value_content)?;

        let next_pub_value_content = read_to_string(&pub_value_path2).await?;
        let next_pub_value: PublicValues = serde_json::from_str(&next_pub_value_content)?;

        // Update public values for the aggregation.
        let agg_public_values = PublicValues {
            roots_before: root_pub_value.roots_before,
            roots_after: next_pub_value.roots_after,
        };
        // We can duplicate the proofs here because the state hasn't mutated.
        let (agg_proof, updated_agg_public_values) = all_circuits.prove_aggregation(
            is_agg1,
            &next_proof,
            is_agg2,
            &root_proof,
            agg_public_values.clone(),
        )?;
        all_circuits.verify_aggregation(&agg_proof)?;

        // write agg_proof write file
        let json_string = serde_json::to_string(&agg_proof)?;
        write_file(&agg_proof_path, json_string.as_bytes()).await?;

        // updated_agg_public_values file
        let json_string = serde_json::to_string(&updated_agg_public_values)?;
        write_file(&agg_pub_value_path, json_string.as_bytes()).await?;

        Ok(())
    }
}
