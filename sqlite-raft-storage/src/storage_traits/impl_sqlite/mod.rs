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
use raft::Result as RaftResult;
use raft::StorageError;
use rusqlite::Transaction;
use self::append::entries_trim_front;
use SqliteStorage;
use storage_traits::StorageMut;
use storage_traits::StorageTestable;

mod append;

impl StorageMut for SqliteStorage {
    fn set_hardstate(&self, hs: HardState) -> RaftResult<()> {
        self.inside_transaction(|tx: &Transaction, core_id: CoreId| {
            SqliteHardState::from(hs).insert_or_replace(tx, core_id)
        })?;

        Ok(())
    }

    fn apply_snapshot(&self, snapshot: Snapshot) -> RaftResult<()> {
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
        })?;

        Ok(())
    }

    fn create_snapshot(&self, _idx: u64, _cs: Option<ConfState>, _data: Vec<u8>) -> RaftResult<()> {
        unimplemented!()
    }

    fn compact(&self, _compact_index: u64) -> RaftResult<()> {
        unimplemented!()
    }

    fn append(&self, entries: &[Entry]) -> RaftResult<()> {
        if entries.is_empty() {
            return Ok(());
        }

        self.inside_transaction(|tx: &Transaction, core_id: CoreId| {
            let current_first_idx = SqliteEntry::first_index(tx, core_id)?;
            let entries_tail = entries_trim_front(entries, current_first_idx);
            let sqlite_entries = SqliteEntries::from(entries_tail.to_vec());
            sqlite_entries.append(tx, core_id)
        })?;

        Ok(())
    }
}

impl StorageTestable for SqliteStorage {
    fn set_entries(&self, entries: &[Entry]) {
        let sqlite_entries = SqliteEntries::from(entries.to_vec());
        self.inside_transaction(|tx: &Transaction, core_id: CoreId| {
            sqlite_entries.replace_all(tx, core_id)
        }).unwrap();
    }

    fn clone_entries(&self) -> Vec<Entry> {
        let sqlite_entries: SqliteEntries = self.inside_transaction(|tx: &Transaction, core_id: CoreId| {
            SqliteEntries::query_all(tx, core_id)
        }).unwrap();

        sqlite_entries.into()
    }
}
