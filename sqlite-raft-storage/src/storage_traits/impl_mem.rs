use raft::eraftpb::ConfState;
use raft::eraftpb::Entry;
use raft::eraftpb::HardState;
use raft::eraftpb::Snapshot;
use raft::Result as RaftResult;
use raft::storage::MemStorage;
use storage_traits::StorageMut;
use storage_traits::StorageTestable;


impl StorageMut for MemStorage {
    fn set_hardstate(&self, hs: HardState) -> RaftResult<()> {
        self.wl().set_hardstate(hs);

        Ok(())
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

impl StorageTestable for MemStorage {
    fn set_entries(&self, entries: &[Entry]) {
        self.wl().set_entries(entries);
    }

    fn clone_entries(&self) -> Vec<Entry> {
        self.rl().clone_entries()
    }
}
