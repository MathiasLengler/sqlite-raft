fn main() {
    eprintln!("{}", ::std::env::current_dir().unwrap().display());

    let proto_root = "proto";

    let includes = [proto_root, "../raft-rs/proto", "../sqlite-requests/proto"];

    for include in includes.iter() {
        println!("cargo:rerun-if-changed={}", include);
    }

    let proto_files = ["helloworld.proto", "raftsqlite.proto"];

    for proto_file in proto_files.iter() {
        println!("cargo:rerun-if-changed={}/{}", proto_root, proto_file);
    }

    let proto_gen_output = "src/proto";

    protoc_grpcio::compile_grpc_protos(
        &proto_files,
        &includes,
        &proto_gen_output,
    ).expect("Failed to compile gRPC definitions!");
}

extern crate protoc_grpcio;
