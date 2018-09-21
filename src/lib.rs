#[macro_use]
extern crate failure;
extern crate raft;
extern crate rusqlite;

use error::Result;
use raft::eraftpb::Entry;
use raft::eraftpb::Snapshot;
use raft::RaftState;
use raft::Result as RaftResult;
use raft::Storage;
use rusqlite::Connection;
use std::path::Path;

mod hard_state;
pub mod error;

// TODO: MemStorageCore -> Tables
// TODO: Test against MemStorageCore
// TODO: use in sqlite-raft

struct SqliteStorage {
    conn: Connection,
}

impl SqliteStorage {
    fn new<P: AsRef<Path>>(path: P) -> Result<SqliteStorage> {
        Ok(SqliteStorage {
            conn: Connection::open(path)?,
        })
    }
}

impl Storage for SqliteStorage {
    fn initial_state(&self) -> RaftResult<RaftState> {
        unimplemented!()
    }

    fn entries(&self, low: u64, high: u64, max_size: u64) -> RaftResult<Vec<Entry>> {
        unimplemented!()
    }

    fn term(&self, idx: u64) -> RaftResult<u64> {
        unimplemented!()
    }

    fn first_index(&self) -> RaftResult<u64> {
        unimplemented!()
    }

    fn last_index(&self) -> RaftResult<u64> {
        unimplemented!()
    }

    fn snapshot(&self) -> RaftResult<Snapshot> {
        unimplemented!()
    }
}