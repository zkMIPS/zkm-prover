fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .type_attribute(".", "#[derive(serde_derive::Serialize)]")
        .type_attribute(".", "#[derive(serde_derive::Deserialize)]")
        .compile(
            &[
                "../proto/src/proto/include/v1/program.proto",
                "../proto/src/proto/stage/v1/stage.proto",
            ],
            &["../proto/src/proto"],
        )?;
    Ok(())
}
