extern crate protoc_grpcio;

use std::env;
use std::path;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    let proto_root = "src/proto";
    println!("cargo:rerun-if-changed={}", proto_root);

    let current_dir = env::current_dir().unwrap();
    eprintln!("current_dir = {:?}", current_dir);

    let out_dir = env::var("OUT_DIR").unwrap();
    eprintln!("out_dir = {:?}", out_dir);

    let output_dir: path::PathBuf = [&out_dir].iter().collect();

    let mod_rs = output_dir.join("mod.rs");
    let mut module = File::create(mod_rs).unwrap();

    module.write_all(
        r#"
pub mod helloworld_grpc;
pub mod helloworld;
"#.as_bytes()
    ).unwrap();

    protoc_grpcio::compile_grpc_protos(
        &["helloworld.proto"],
        &[proto_root],
        &output_dir
    ).expect("Failed to compile gRPC definitions!");
}
