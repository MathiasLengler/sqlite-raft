extern crate protobuf_codegen_pure;

fn main() {
    protobuf_codegen_pure::run(protobuf_codegen_pure::Args {
        out_dir: "src/proto",
        input: &["proto/sqlite_requests.proto"],
        includes: &["proto"],
        customize: protobuf_codegen_pure::Customize {
            ..Default::default()
        },
        ..Default::default()
    }).expect("protoc");
}
