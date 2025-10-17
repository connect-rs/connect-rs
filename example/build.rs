fn main() {
    std::fs::create_dir_all("src/generated").unwrap();

    prost_build::Config::new()
        .out_dir("src/generated")
        .file_descriptor_set_path("src/generated/file_descriptor_set.bin")
        .type_attribute(".", "#[allow(dead_code)]")
        .compile_protos(&["proto/greet/v1/example.proto"], &["proto"])
        .unwrap();

    std::process::Command::new("protoc")
        .args([
            "--connect-rust_out=src",
            "--proto_path=proto",
            "proto/greet/v1/example.proto",
        ])
        .status()
        .expect("Failed to run protoc");
}
