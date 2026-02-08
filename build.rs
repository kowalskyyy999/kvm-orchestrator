fn main() -> Result<(), Box<dyn std::error::Error>> {
    // tonic_build::compile_protos("protos/user_master.proto").unwrap();

    let mut config = prost_build::Config::new();
    config.protoc_arg("--experimental_allow_proto3_optional");

    tonic_build::configure()
        .build_server(true)
        .compile_with_config(config, &["protos/virt_service.proto"], &["protos/"])
        .unwrap();
    Ok(())
}
