fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .type_attribute(".", "#[derive(serde_derive::Serialize)]")
        .compile(
            &[
                "../proto/src/proto/prover/v1/prover.proto",
                "../proto/src/proto/include/v1/program.proto",
            ],
            &["../proto/src/proto"],
        )?;
    Ok(())
}
