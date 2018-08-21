extern crate protobuf;
extern crate grpcio;
extern crate futures;

pub mod log_util;

pub mod proto_gen {
    include!(concat!(env!("OUT_DIR"), "/mod.rs"));
}
