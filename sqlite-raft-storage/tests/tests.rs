extern crate raft;
extern crate protobuf;
extern crate sqlite_raft_storage;
extern crate sqlite_test_utils;

#[macro_use]
extern crate model;
#[macro_use]
extern crate proptest;

mod mem_storage_tests;
mod utils;
mod model_tests;

// TODO: implement model test Mem vs Sqlite storage
