use storage_traits::StorageMut;
use raft::storage::MemStorage;
use raft::eraftpb::HardState;
use raft::eraftpb::Entry;
use raft::Error as RaftError;
use raft::Result as RaftResult;
use raft::eraftpb::Snapshot;
use raft::eraftpb::ConfState;
use raft::Storage;


impl StorageMut for MemStorage {
    type StorageError = RaftError;

    fn set_hardstate(&self, hs: HardState) {
        self.wl().set_hardstate(hs);
    }

    fn apply_snapshot(&self, snapshot: Snapshot) -> RaftResult<()> {
        self.wl().apply_snapshot(snapshot)
    }

    fn create_snapshot(&self, idx: u64, cs: Option<ConfState>, data: Vec<u8>) -> RaftResult<()> {
        self.wl().create_snapshot(
            idx,
            cs,
            data,
        )?;

        Ok(())
    }

    fn compact(&self, compact_index: u64) -> RaftResult<()> {
        self.wl().compact(compact_index)
    }

    fn append(&self, entries: &[Entry]) -> RaftResult<()> {
        self.wl().append(entries)
    }
}
