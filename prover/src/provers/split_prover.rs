use super::Prover;
use crate::contexts::SplitContext;
use anyhow::Ok;
use anyhow::Result;

use elf::{endian::AnyEndian, ElfBytes};
use num::ToPrimitive;
use std::fs;

use zkm::mips_emulator::state::{InstrumentedState, State, SEGMENT_STEPS};
use zkm::mips_emulator::utils::get_block_path;

#[derive(Default)]
pub struct SplitProver {}

impl SplitProver {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Prover<SplitContext> for SplitProver {
    fn prove(&self, ctx: &SplitContext) -> Result<()> {
        // 1. split ELF into segs
        let basedir = ctx.basedir.clone();
        let elf_path = ctx.elf_path.clone();
        let block_no = ctx.block_no.to_string();
        let seg_path = ctx.seg_path.clone();
        let seg_size = ctx.seg_size.to_usize().expect("u32->usize failed");
        let args = "".to_string();

        let data = fs::read(elf_path)?;
        let file_result = ElfBytes::<AnyEndian>::minimal_parse(data.as_slice());
        match file_result {
            core::result::Result::Ok(file) => {
                let (mut state, _) = State::load_elf(&file);
                state.patch_go(&file);
                state.patch_stack(&args);

                let block_path = get_block_path(&basedir, &block_no, "");
                state.load_input(&block_path);


                let mut instrumented_state = InstrumentedState::new(state, block_path);
                instrumented_state.split_segment(false, &seg_path);
                let mut segment_step: usize = seg_size;
                loop {
                    if instrumented_state.state.exited {
                        break;
                    }
                    instrumented_state.step();
                    segment_step -= 1;
                    if segment_step == 0 {
                        segment_step = seg_size;
                        instrumented_state.split_segment(true, &seg_path);
                    }
                }
                instrumented_state.split_segment(true, &seg_path);
            }
            Err(_) => {
            }
        }        
        Ok(())
    }
}