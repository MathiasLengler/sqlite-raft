use error::{Error, Result};
use model::core::CoreId;
use model::entry::SqliteEntries;
use model::entry::SqliteEntry;
use model::hard_state::SqliteHardState;
use model::snapshot::SqliteSnapshot;
use model::snapshot::SqliteSnapshotMetadata;
use raft::eraftpb::ConfState;
use raft::eraftpb::Entry;
use raft::eraftpb::HardState;
use raft::eraftpb::Snapshot;
use raft::Error as RaftError;
use raft::StorageError;
use rusqlite::Transaction;
use SqliteStorage;
use storage_traits::StorageMut;
use storage_traits::StorageTestable;
use self::append::entries_trim_front;

mod append;

impl StorageMut for SqliteStorage {
    type StorageError = Error;

    fn set_hardstate(&self, hs: HardState) -> Result<()> {
        self.inside_transaction(|tx: &Transaction, core_id: CoreId| {
            SqliteHardState::from(hs).insert_or_replace(tx, core_id)
        })
    }

    fn apply_snapshot(&self, snapshot: Snapshot) -> Result<()> {
        self.inside_transaction(|tx: &Transaction, core_id: CoreId| {
            // handle check for old snapshot being applied
            let entry = {
                let old_metadata = SqliteSnapshotMetadata::query(tx, core_id)?;
                let new_metadata = snapshot.get_metadata();

                if old_metadata.index as u64 >= new_metadata.get_index() {
                    return Err(RaftError::Store(StorageError::SnapshotOutOfDate).into());
                }

                let mut entry = Entry::new();
                entry.set_term(new_metadata.get_term());
                entry.set_index(new_metadata.get_index());
                entry
            };

            let sqlite_entries: SqliteEntries = vec![entry].into();
            sqlite_entries.replace_all(tx, core_id)?;

            let sqlite_snapshot: SqliteSnapshot = snapshot.into();
            sqlite_snapshot.insert_or_replace(tx, core_id)?;

            Ok(())
        })
    }

    fn create_snapshot(&self, idx: u64, cs: Option<ConfState>, data: Vec<u8>) -> Result<()> {
        unimplemented!()
    }

    fn compact(&self, compact_index: u64) -> Result<()> {
        unimplemented!()
    }

    fn append(&self, entries: &[Entry]) -> Result<()> {
        if entries.is_empty() {
            return Ok(());
        }

        self.inside_transaction(|tx: &Transaction, core_id: CoreId| {
            let current_first_idx = SqliteEntry::first_index(tx, core_id)?;
            let entries_tail = entries_trim_front(entries, current_first_idx);
            let sqlite_entries = SqliteEntries::from(entries_tail.to_vec());
            sqlite_entries.append(tx, core_id)
        })
    }
}

impl StorageTestable for SqliteStorage {
    fn set_entries(&self, entries: &[Entry]) {
        unimplemented!()
    }

    fn clone_entries(&self) -> Vec<Entry> {
        unimplemented!()
    }
}
