pub mod contexts;
pub mod executor;
pub mod pipeline;
pub mod provers;

#[cfg(feature = "gpu")]
pub fn init_stark_op_stream_simple() {
    use plonky2::field::abstract_field::{get_ops_stream, get_ops_stream_simple, AbstractFieldForVec, SimpleOpsNode};
    let ops_vec_1: Option<Vec<SimpleOpsNode>> = {
        let file_path = "/mnt_zkm/app/mytest_get_opstreams/log_simplify_arithmetic.txt";
        match get_ops_stream_simple(file_path) {
            Ok(Some(ops)) => Some(ops),
            Ok(None) => None,
            Err(_) => None,
        }
    };
    let ops_vec_2: Option<Vec<SimpleOpsNode>> = {
        let file_path = "/mnt_zkm/app/mytest_get_opstreams/log_simplify_cpu.txt";
        match get_ops_stream_simple(file_path) {
            Ok(Some(ops)) => Some(ops),
            Ok(None) => None,
            Err(_) => None,
        }
    };
    let ops_vec_3: Option<Vec<SimpleOpsNode>> = {
        let file_path = "/mnt_zkm/app/mytest_get_opstreams/log_simplify_poseidon.txt";
        match get_ops_stream_simple(file_path) {
            Ok(Some(ops)) => Some(ops),
            Ok(None) => None,
            Err(_) => None,
        }
    };
    let ops_vec_4: Option<Vec<SimpleOpsNode>> = {
        let file_path = "/mnt_zkm/app/mytest_get_opstreams/log_simplify_poseidonsponge.txt";
        match get_ops_stream_simple(file_path) {
            Ok(Some(ops)) => Some(ops),
            Ok(None) => None,
            Err(_) => None,
        }
    };
    let ops_vec_5: Option<Vec<SimpleOpsNode>> = {
        let file_path = "/mnt_zkm/app/mytest_get_opstreams/log_simplify_logic.txt";
        match get_ops_stream_simple(file_path) {
            Ok(Some(ops)) => Some(ops),
            Ok(None) => None,
            Err(_) => None,
        }
    };
    let ops_vec_6: Option<Vec<SimpleOpsNode>> = {
        let file_path = "/mnt_zkm/app/mytest_get_opstreams/log_simplify_mem.txt";
        match get_ops_stream_simple(file_path) {
            Ok(Some(ops)) => Some(ops),
            Ok(None) => None,
            Err(_) => None,
        }
    };
    let mut abstract_field_vec = plonky2::field::abstract_field::SIMPLE_STARKS_ABSTRACT_FIELD_VEC
        .lock()
        .unwrap();
    abstract_field_vec.push(ops_vec_1.clone());
    abstract_field_vec.push(ops_vec_2.clone());
    abstract_field_vec.push(ops_vec_3.clone());
    abstract_field_vec.push(ops_vec_4.clone());
    abstract_field_vec.push(ops_vec_5.clone());
    abstract_field_vec.push(ops_vec_6.clone());
    log::info!("streams_len: {}", ops_vec_1.unwrap().len());
    log::info!("streams_len: {}", ops_vec_2.unwrap().len());
    log::info!("streams_len: {}", ops_vec_3.unwrap().len());
    log::info!("streams_len: {}", ops_vec_4.unwrap().len());
    log::info!("streams_len: {}", ops_vec_5.unwrap().len());
    log::info!("streams_len: {}", ops_vec_6.unwrap().len());
}