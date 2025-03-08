#[allow(clippy::module_inception)]
pub mod prover_service {
    pub mod v1 {
        tonic::include_proto!("prover.v1");
    }
}

pub mod includes {
    pub mod v1 {
        tonic::include_proto!("includes.v1");
    }
}
pub mod stage_service {
    pub mod v1 {
        tonic::include_proto!("stage.v1");
    }
}
