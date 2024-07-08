use crate::split_context::SplitContext;
use common::file;
use elf::{endian::AnyEndian, ElfBytes};
use num::ToPrimitive;
use zkm_prover::mips_emulator::state::{InstrumentedState, State};
use zkm_prover::mips_emulator::utils::get_block_path;

#[derive(Default)]
pub struct Executor {}

impl Executor {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Executor {
    pub fn split(&self, ctx: &SplitContext) -> std::result::Result<bool, String> {
        // 1. split ELF into segs
        let basedir = ctx.basedir.clone();
        let elf_path = ctx.elf_path.clone();
        let block_no = ctx.block_no.to_string();
        let seg_path = ctx.seg_path.clone();
        let seg_size = ctx.seg_size.to_usize().expect("u32->usize failed");
        let args = ctx.args.split_whitespace().collect();

        log::info!("split {} load elf file", elf_path);
        let data = file::new(&elf_path).read();
        let mut block_path = get_block_path(&basedir, &block_no, "");
        let input_path = if block_path.ends_with('/') {
            format!("{}input", block_path)
        } else {
            format!("{}/input", block_path)
        };

        if let core::result::Result::Ok(data) = data {
            let file_result = ElfBytes::<AnyEndian>::minimal_parse(data.as_slice());
            match file_result {
                core::result::Result::Ok(file) => {
                    let (mut state, _) = State::load_elf(&file);
                    state.patch_elf(&file);
                    state.patch_stack(args);

                    let block_no = block_no.parse::<_>().unwrap_or(0);
                    if block_no > 0 {
                        log::info!("split set input data {}", input_path);
                        let input_data = file::new(&input_path).read().unwrap();
                        state
                            .memory
                            .set_memory_range(0x30000000, Box::new(input_data.as_slice()))
                            .expect("set memory range failed");
                    } else {
                        block_path = "".to_string();
                    }

                    let mut instrumented_state = InstrumentedState::new(state, block_path);
                    let seg_path_clone = seg_path.clone();
                    file::new(&seg_path_clone).create_dir_all().unwrap();
                    let new_write = |_: &str| -> Option<std::fs::File> { None };
                    instrumented_state.split_segment(false, &seg_path_clone, new_write);
                    let mut segment_step: usize = seg_size;

                    let new_write =
                        |name: &str| -> Option<Box<dyn std::io::Write>> { Some(file::new(name)) };
                    loop {
                        if instrumented_state.state.exited {
                            break;
                        }
                        instrumented_state.step();
                        segment_step -= 1;
                        if segment_step == 0 {
                            segment_step = seg_size;
                            instrumented_state.split_segment(true, &seg_path_clone, new_write);
                        }
                    }
                    instrumented_state.split_segment(true, &seg_path_clone, new_write);
                    return Ok(true);
                }
                Err(e) => {
                    log::error!("split minimal_parse error {}", e.to_string());
                    return Err(e.to_string());
                }
            }
        }
        Ok(false)
    }
}
