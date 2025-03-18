fn main() -> Result<(), Box<dyn std::error::Error>> {
    if cfg!(all(feature = "prover", feature = "prover_v2")) {
        panic!("Cannot enable both prover and prover_v2 features!");
    }

    if cfg!(not(any(feature = "prover", feature = "prover_v2"))) {
        panic!("Either prover or prover_v2 must be enabled!");
    }

    tonic_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .type_attribute(
            ".",
            "#[derive(serde_derive::Serialize, serde_derive::Deserialize)]",
        )
        .compile(
            &[
                "../proto/src/proto/prover/v1/prover.proto",
                "../proto/src/proto/stage/v1/stage.proto",
                "../proto/src/proto/include/v1/includes.proto",
            ],
            &["../proto/src/proto"],
        )?;
    Ok(())
}
