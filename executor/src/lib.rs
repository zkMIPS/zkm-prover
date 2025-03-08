pub mod executor;
pub mod split_context;

pub mod program {
    pub mod v1 {
        tonic::include_proto!("program.v1");
    }
}
