use crate::split_context::SplitContext;
use crate::traces_util::TracesUtil;
use common::file;
use elf::{endian::AnyEndian, ElfBytes};
use num::ToPrimitive;
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::plonk::config::PoseidonGoldilocksConfig;
use zkm_emulator::state::{InstrumentedState, State};
use zkm_emulator::utils::get_block_path;
use zkm_prover::generation::state::{AssumptionReceipts, Receipt};

#[derive(Default)]
pub struct Executor {}

impl Executor {
    pub fn new() -> Self {
        Self::default()
    }
}

type F = GoldilocksField;
const D: usize = 2;
type C = PoseidonGoldilocksConfig;

impl Executor {
    pub fn split(&self, ctx: &SplitContext) -> std::result::Result<u64, String> {
        // 1. split ELF into segs
        let basedir = ctx.basedir.clone();
        let elf_path = ctx.elf_path.clone();
        let block_no = ctx.block_no.to_string();
        let seg_path = ctx.seg_path.clone();
        let seg_size = ctx.seg_size.to_usize().expect("u32->usize failed");
        let mut args: Vec<&str> = ctx.args.split_whitespace().collect();
        if args.len() > 2 {
            args.truncate(2);
        }

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
                    let mut state = State::load_elf(&file);
                    state.patch_elf(&file);
                    // state.patch_stack(args);
                    state.patch_stack(vec![]);
                    // public_input_stream
                    if !ctx.public_input_path.is_empty() {
                        let data = file::new(&ctx.public_input_path)
                            .read()
                            .expect("read public_input_stream failed");
                        state.input_stream.push(data.clone());
                        log::info!("split set public_input data {}", data.len());

                        // private_input_stream
                        if !ctx.private_input_path.is_empty() {
                            let data = file::new(&ctx.private_input_path)
                                .read()
                                .expect("read private_input_stream failed");
                            state.input_stream.push(data.clone());
                            log::info!("split set private_input data {}", data.len());
                        }
                    }

                    if !ctx.receipt_inputs_path.is_empty() {
                        let data = file::new(&ctx.receipt_inputs_path)
                            .read()
                            .expect("read receipt_inputs_stream failed");
                        let receipt_inputs = bincode::deserialize::<Vec<Vec<u8>>>(&data)
                            .expect("deserialize receipt_inputs_stream failed");
                        for receipt_input in receipt_inputs.iter() {
                            state.input_stream.push(receipt_input.clone());
                            log::info!("split set receipt_inputs data {}", data.len());
                        }
                    }

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

                    let mut receipts: AssumptionReceipts<F, C, D> = vec![];
                    if !ctx.receipts_path.is_empty() {
                        let data = file::new(&ctx.receipts_path)
                            .read()
                            .expect("read receipts_path failed");
                        let receipt_datas = bincode::deserialize::<Vec<Vec<u8>>>(&data)
                            .expect("deserialize receipts failed");
                        for receipt_data in receipt_datas.iter() {
                            let receipt: Receipt<F, C, D> =
                                bincode::deserialize(receipt_data).map_err(|e| e.to_string())?;
                            receipts.push(receipt.into());
                            log::info!("prove set receipts {:?}", receipt_data.len());
                        }
                    }

                    let mut instrumented_state = InstrumentedState::new(state, block_path.clone());
                    let seg_path_clone = seg_path.clone();
                    file::new(&seg_path_clone).create_dir_all().unwrap();
                    let new_write = |_: &str| -> Option<std::fs::File> { None };
                    instrumented_state.split_segment(false, &seg_path_clone, new_write);

                    let new_write =
                        |name: &str| -> Option<Box<dyn std::io::Write>> { Some(file::new(name)) };
                    loop {
                        if instrumented_state.state.exited {
                            break;
                        }
                        let cycles = instrumented_state.step();
                        if cycles > (seg_size as isize - 1) as u64 {
                            instrumented_state.split_segment(true, &seg_path_clone, new_write);
                            let pre_segment_id = Self::check_and_re_split(
                                ctx,
                                receipts.clone(),
                                instrumented_state.pre_segment_id - 1,
                                seg_size,
                                &block_path,
                            )?;
                            instrumented_state.pre_segment_id = pre_segment_id;
                        }
                    }
                    instrumented_state.split_segment(true, &seg_path_clone, new_write);
                    let pre_segment_id = Self::check_and_re_split(
                        ctx,
                        receipts.clone(),
                        instrumented_state.pre_segment_id - 1,
                        seg_size,
                        &block_path,
                    )?;
                    instrumented_state.pre_segment_id = pre_segment_id;

                    log::info!(
                        "Split done {} : {} - {}",
                        instrumented_state.state.total_step,
                        instrumented_state.state.total_cycle,
                        instrumented_state.pre_segment_id
                    );
                    instrumented_state.dump_memory();
                    // write public_values_stream
                    let _ = file::new(&ctx.output_path)
                        .write(&instrumented_state.state.public_values_stream)
                        .unwrap();
                    return Ok(instrumented_state.state.total_step);
                }
                Err(e) => {
                    log::error!("split minimal_parse error {}", e.to_string());
                    return Err(e.to_string());
                }
            }
        }
        Ok(0)
    }

    fn check_and_re_split(
        ctx: &SplitContext,
        receipts: AssumptionReceipts<F, C, D>,
        pre_segment_id: u32,
        seg_size: usize,
        block_path: &str,
    ) -> std::result::Result<u32, String> {
        let seg_path = ctx.seg_path.clone();
        let seg_file = format!("{}/{}", seg_path, pre_segment_id);
        let traces_info = TracesUtil::get_traces_len(ctx, receipts.clone(), &seg_file)
            .map_err(|e| e.to_string())?;
        let max_trace_len = traces_info.iter().max().unwrap();
        if *max_trace_len > seg_size {
            let seg_size = seg_size / (*max_trace_len / seg_size + 1);
            let (state, final_step) = Self::load_segment(&seg_file);
            file::new(&seg_file).remove().unwrap();
            let mut instrumented_state = InstrumentedState::new(state, block_path.to_string());
            log::debug!("start pc: {:X} {}", instrumented_state.state.pc, final_step);
            let new_writer = |_: &str| -> Option<std::fs::File> { None };
            instrumented_state.split_segment(false, &seg_path, new_writer);
            instrumented_state.pre_segment_id = pre_segment_id;
            let new_writer =
                |name: &str| -> Option<Box<dyn std::io::Write>> { Some(file::new(name)) };
            loop {
                let cycles = instrumented_state.step();
                if instrumented_state.state.total_step + instrumented_state.state.step == final_step
                {
                    break;
                }
                if cycles > (seg_size as isize - 1) as u64 {
                    instrumented_state.split_segment(true, &seg_path, new_writer);
                    let pre_segment_id = Self::check_and_re_split(
                        ctx,
                        receipts.clone(),
                        instrumented_state.pre_segment_id,
                        seg_size,
                        block_path,
                    )?;
                    instrumented_state.pre_segment_id = pre_segment_id;
                    log::debug!(
                        "Split at {} : {} into {}",
                        instrumented_state.state.total_step,
                        instrumented_state.state.total_cycle,
                        instrumented_state.pre_segment_id
                    );
                }
            }
            instrumented_state.split_segment(true, &seg_path, new_writer);
            let pre_segment_id = Self::check_and_re_split(
                ctx,
                receipts.clone(),
                instrumented_state.pre_segment_id,
                seg_size,
                block_path,
            )?;
            instrumented_state.pre_segment_id = pre_segment_id;
            log::info!(
                "Split done {} : {} into {}",
                instrumented_state.state.total_step,
                instrumented_state.state.total_cycle,
                instrumented_state.pre_segment_id
            );
            Ok(instrumented_state.pre_segment_id)
        } else {
            Ok(pre_segment_id + 1)
        }
    }

    pub fn load_segment(seg_file: &str) -> (Box<State>, u64) {
        State::load_seg(seg_file)
    }
}
