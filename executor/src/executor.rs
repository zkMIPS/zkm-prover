use crate::split_context::SplitContext;
use common::file::{create_dir_all, read, write_file};
use elf::{endian::AnyEndian, ElfBytes};
use num::ToPrimitive;
use zkm::mips_emulator::state::{InstrumentedState, State};
use zkm::mips_emulator::utils::get_block_path;

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
        let args = "".to_string();

        let data = read(&elf_path);
        let block_path = get_block_path(&basedir, &block_no, "");
        let input_path = if block_path.ends_with('/') {
            format!("{}input", block_path)
        } else {
            format!("{}/input", block_path)
        };
        let input_data = read(&input_path).unwrap();
        if let core::result::Result::Ok(data) = data {
            let file_result = ElfBytes::<AnyEndian>::minimal_parse(data.as_slice());
            match file_result {
                core::result::Result::Ok(file) => {
                    let (mut state, _) = State::load_elf(&file);
                    state.patch_go(&file);
                    state.patch_stack(&args);

                    state
                        .memory
                        .set_memory_range(0x30000000, Box::new(input_data.as_slice()))
                        .expect("set memory range failed");

                    let mut instrumented_state = InstrumentedState::new(state, block_path);
                    // proof is false would not return segments
                    let seg_path_clone = seg_path.clone();
                    create_dir_all(&seg_path_clone).unwrap();

                    instrumented_state.get_split_segments(false);
                    let mut segment_step: usize = seg_size;
                    loop {
                        if instrumented_state.state.exited {
                            break;
                        }
                        instrumented_state.step();
                        segment_step -= 1;
                        if segment_step == 0 {
                            segment_step = seg_size;
                            let segments = instrumented_state.get_split_segments(true);
                            for segment in segments {
                                let segment_path = format!("{}/{}", seg_path, segment.segment_id);
                                let data = serde_json::to_vec(&segment).unwrap();
                                write_file(&segment_path, &data).unwrap();
                            }
                        }
                    }
                    let segments = instrumented_state.get_split_segments(true);
                    for segment in segments {
                        let segment_path = format!("{}/{}", seg_path, segment.segment_id);
                        let data = serde_json::to_vec(&segment).unwrap();
                        write_file(&segment_path, &data).unwrap();
                    }
                    return Ok(true);
                }
                Err(e) => {
                    return Err(e.to_string());
                }
            }
        }
        Ok(false)
    }
}
