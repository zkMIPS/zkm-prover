fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile(
            &["../proto/src/proto/include/v1/program.proto"],
            &["../proto/src/proto"],
        )?;
    Ok(())
}
