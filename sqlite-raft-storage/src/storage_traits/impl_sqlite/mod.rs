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

            let sqlite_entries = SqliteEntries::try_from_entry_vec(vec![entry])?;
            sqlite_entries.replace_all(tx, core_id)?;

            let sqlite_snapshot: SqliteSnapshot = snapshot.into();
            sqlite_snapshot.insert_or_replace(tx, core_id)?;

            Ok(())
        })?;

        Ok(())
    }

    fn create_snapshot(&self, idx: u64, cs: Option<ConfState>, data: Vec<u8>) -> RaftResult<()> {
        self.inside_transaction(|tx: &Transaction, core_id: CoreId| {
            let sqlite_snapshot = SqliteSnapshot::query(tx, core_id)?;
            let mut snapshot: Snapshot = sqlite_snapshot.into();

            if idx <= snapshot.get_metadata().get_index() {
                return Err(RaftError::Store(StorageError::SnapshotOutOfDate).into());
            }

            let last_index = SqliteEntry::last_index(tx, core_id)?;

            if idx > last_index {
                // TODO: return error
                panic!(
                    "snapshot {} is out of bound lastindex({})",
                    idx,
                    last_index
                )
            }

            snapshot.mut_metadata().set_index(idx);

            let entry_idx: Entry = SqliteEntry::query(tx, core_id, idx)?.into();

            snapshot
                .mut_metadata()
                .set_term(entry_idx.get_term());
            if let Some(cs) = cs {
                snapshot.mut_metadata().set_conf_state(cs)
            }
            snapshot.set_data(data);

            let sqlite_snapshot: SqliteSnapshot = snapshot.into();
            sqlite_snapshot.insert_or_replace(tx, core_id)?;

            Ok(())
        })?;

        Ok(())
    }

    /// Discards all log entries prior to compact_index.
    /// It is the application's responsibility to not attempt to compact an index
    /// greater than RaftLog.applied.
    fn compact(&self, compact_index: u64) -> RaftResult<()> {
        self.inside_transaction(|tx: &Transaction, core_id: CoreId| {
            let first_index = SqliteEntry::first_index(tx, core_id)?;
            let last_index = SqliteEntry::last_index(tx, core_id)?;

            if compact_index <= first_index {
                return Err(RaftError::Store(StorageError::Compacted).into());
            }
            if compact_index > last_index {
                // TODO: return error
                panic!(
                    "compact {} is out of bound lastindex({})",
                    compact_index,
                    last_index
                )
            }


            let compact_entry = SqliteEntry::query(tx, core_id, compact_index)?;
            compact_entry.truncate_left(tx, core_id)?;

            Ok(())
        })?;

        Ok(())
    }

    fn append(&self, entries: &[Entry]) -> RaftResult<()> {
        if entries.is_empty() {
            return Ok(());
        }

        self.inside_transaction(|tx: &Transaction, core_id: CoreId| {
            let current_first_idx = SqliteEntry::first_index(tx, core_id)?;
            let entries_tail = entries_trim_front(entries, current_first_idx);
            let sqlite_entries = SqliteEntries::try_from_entry_vec(entries_tail.to_vec())?;
            sqlite_entries.append(tx, core_id)
        })?;

        Ok(())
    }
}

impl StorageTestable for SqliteStorage {
    fn set_entries(&self, entries: &[Entry]) {
        let sqlite_entries = SqliteEntries::try_from_entry_vec(entries.to_vec()).unwrap();
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
