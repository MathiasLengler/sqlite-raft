use raft::eraftpb::ConfState;
use raft::eraftpb::Entry;
use raft::eraftpb::HardState;
use raft::eraftpb::Snapshot;
use raft::Result as RaftResult;
use raft::Storage;
use std::fmt::Debug;

mod impl_mem;
mod impl_sqlite;

pub trait StorageMut: Storage {
    fn set_hardstate(&self, hs: HardState) -> RaftResult<()>;

    fn apply_snapshot(&self, snapshot: Snapshot) -> RaftResult<()>;

    fn create_snapshot(
        &self,
        idx: u64,
        cs: Option<ConfState>,
        data: Vec<u8>,
    ) -> RaftResult<()>;

    fn compact(&self, compact_index: u64) -> RaftResult<()>;

    fn append(&self, entries: &[Entry]) -> RaftResult<()>;
}

// TODO: cfg test + impls
pub trait StorageTestable: StorageMut + Debug {
    fn set_entries(&self, entries: &[Entry]);

    fn clone_entries(&self) -> Vec<Entry>;
}
