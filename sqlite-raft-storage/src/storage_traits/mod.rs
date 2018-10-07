use raft::eraftpb::ConfState;
use raft::eraftpb::Entry;
use raft::eraftpb::HardState;
use raft::eraftpb::Snapshot;
use raft::Storage;

mod impl_mem;
mod impl_sqlite;

pub trait StorageMut: Storage {
    type StorageError;

    fn set_hardstate(&self, hs: HardState) -> Result<(), Self::StorageError>;

    fn apply_snapshot(&self, snapshot: Snapshot) -> Result<(), Self::StorageError>;

    fn create_snapshot(
        &self,
        idx: u64,
        cs: Option<ConfState>,
        data: Vec<u8>,
    ) -> Result<(), Self::StorageError>;

    fn compact(&self, compact_index: u64) -> Result<(), Self::StorageError>;

    fn append(&self, entries: &[Entry]) -> Result<(), Self::StorageError>;
}

pub trait StorageTestable {
    fn set_entries(&self, entries: &[Entry]);

    fn clone_entries(&self) -> Vec<Entry>;
}
