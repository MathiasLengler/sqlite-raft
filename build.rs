extern crate protoc_rust;

fn main() {
    protoc_rust::run(protoc_rust::Args {
        out_dir: "src/proto",
        input: &["proto/sqlite_requests.proto"],
        customize: protoc_rust::Customize {
            ..Default::default()
        },
        ..Default::default()
    }).expect("protoc");
}
