extern crate raft;
extern crate rusqlite;

use raft::eraftpb::Entry;
use raft::eraftpb::Snapshot;
use raft::Error as RaftError;
use raft::RaftState;
use raft::Storage;
use rusqlite::Connection;


// TODO: MemStorageCore -> Tables
// TODO: Test against MemStorageCore
// TODO: use in sqlite-raft

struct SqliteStorage {
    conn: Connection,
}

impl Storage for SqliteStorage {
    fn initial_state(&self) -> Result<RaftState, RaftError> {
        unimplemented!()
    }

    fn entries(&self, low: u64, high: u64, max_size: u64) -> Result<Vec<Entry>, RaftError> {
        unimplemented!()
    }

    fn term(&self, idx: u64) -> Result<u64, RaftError> {
        unimplemented!()
    }

    fn first_index(&self) -> Result<u64, RaftError> {
        unimplemented!()
    }

    fn last_index(&self) -> Result<u64, RaftError> {
        unimplemented!()
    }

    fn snapshot(&self) -> Result<Snapshot, RaftError> {
        unimplemented!()
    }
}