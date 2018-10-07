extern crate model;
extern crate raft;
extern crate protobuf;
extern crate sqlite_raft_storage;

mod mem_storage_tests;

// TODO: refactor MemStorage tests to run against the Storage traits
// TODO: run test against both storage implementations
// TODO: implement model test Mem vs Sqlite storage
