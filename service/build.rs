fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().protoc_arg("--experimental_allow_proto3_optional").compile(&["proto/src/proto/prover/v1/prover.proto"], &["proto/src/proto"])?;
    tonic_build::configure().protoc_arg("--experimental_allow_proto3_optional").compile(&["proto/src/proto/stage/v1/stage.proto"], &["proto/src/proto"])?;
    Ok(())
}
