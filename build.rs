extern crate protoc_grpcio;

fn main() {
    let proto_root = "src/proto";
    println!("cargo:rerun-if-changed={}", proto_root);

    let proto_files = ["helloworld.proto", "eraftpb.proto", "raftsqlite.proto"];

    for proto_file in proto_files.iter() {
        println!("cargo:rerun-if-changed={}/{}", proto_root, proto_file);
    }

    let proto_gen_output = "src/proto_gen";

    protoc_grpcio::compile_grpc_protos(
        &proto_files,
        &[proto_root],
        &proto_gen_output,
    ).expect("Failed to compile gRPC definitions!");

}
