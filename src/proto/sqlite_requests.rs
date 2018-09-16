// Inject generated protobuf code from another crate to be used by local grpc service.
// Must be generated with the same rust-protobuf version across crates.
pub use sqlite_requests::proto::*;
