#[macro_use]
extern crate failure;
extern crate protobuf;
extern crate raft;
extern crate rusqlite;

use debug::SqliteStorageDebugView;
use error::Result;
use model::core::CoreId;
use model::entry::SqliteEntries;
use model::entry::SqliteEntry;
use model::hard_state::SqliteHardState;
use model::snapshot::node::SqliteConfState;
use model::snapshot::SqliteSnapshot;
use raft::eraftpb::Entry;
use raft::eraftpb::Snapshot;
use raft::RaftState;
use raft::Result as RaftResult;
use raft::Storage;
use rusqlite::Connection;
use rusqlite::Transaction;
use std::fmt;
use std::fmt::Debug;
use std::path::Path;
use std::sync::RwLock;
use rusqlite::NO_PARAMS;

pub(crate) mod model;
pub mod storage_traits;
pub mod error;
pub mod debug;

// TODO: use in sqlite-raft-node
// TODO: TryFrom for "as" casts and proto conversions (take_)
// TODO: persist current user db index (for user-db thread)
// TODO: refactor (tx, core_id) to single parameter (tx_ctx | tx_storage)

// TODO: add conversion of SqliteStorage -> MemStorageCore
// could be used for model testing (equality checking)
pub struct SqliteStorage {
    conn: RwLock<Connection>,
    id: CoreId,
}


impl SqliteStorage {
    const SQL_ON_OPEN: &'static str =
        include_str!("../res/sql/on_open.sql");
    const SQL_EXISTS: &'static str =
        include_str!("../res/sql/exists.sql");
    const SQL_CREATE_TABLES: &'static str =
        include_str!("../res/sql/create_tables.sql");

    pub fn open<P: AsRef<Path>>(path: P) -> Result<SqliteStorage> {
        let storage = SqliteStorage {
            conn: RwLock::new(Connection::open(path)?),
            id: 0.into(),
        };

        storage.init()?;

        Ok(storage)
    }

    fn init(&self) -> Result<()> {
        self.inside_transaction(|tx: &Transaction, core_id: CoreId| {
            tx.execute_batch(SqliteStorage::SQL_ON_OPEN)?;

            SqliteStorage::create_tables_if_not_exists(tx)?;

            if !core_id.exists(tx)? {
                core_id.insert(tx)?;

                SqliteHardState::default().insert_or_replace(tx, core_id)?;
                SqliteSnapshot::default().insert_or_replace(tx, core_id)?;
                SqliteEntries::default().replace_all(tx, core_id)?;
            }

            Ok(())
        })
    }

    fn create_tables_if_not_exists(tx: &Transaction) -> Result<()> {
        let mut stmt = tx.prepare(SqliteStorage::SQL_EXISTS)?;
        if !stmt.exists(NO_PARAMS)? {
            tx.execute_batch(SqliteStorage::SQL_CREATE_TABLES)?;
        }
        Ok(())
    }

    fn inside_transaction<T>(&self, f: impl FnOnce(&Transaction, CoreId) -> Result<T>) -> Result<T> {
        // TODO: handle poisoned lock
        let mut wl_conn = self.conn.write().unwrap();

        let tx = wl_conn.transaction()?;

        let res = f(&tx, self.id)?;

        tx.commit()?;

        Ok(res)
    }

    fn debug_view(&self) -> SqliteStorageDebugView {
        SqliteStorageDebugView::from(self)
    }
}

impl Storage for SqliteStorage {
    fn initial_state(&self) -> RaftResult<RaftState> {
        let raft_state = self.inside_transaction(|tx: &Transaction, core_id: CoreId| {
            Ok(RaftState {
                hard_state: SqliteHardState::query(tx, core_id)?.into(),
                conf_state: SqliteConfState::query(tx, core_id)?.into(),
            })
        })?;

        Ok(raft_state)
    }

    fn entries(&self, low: u64, high: u64, max_size: u64) -> RaftResult<Vec<Entry>> {
        use raft::util::limit_size;

        let sqlite_entries: SqliteEntries = self.inside_transaction(|tx: &Transaction, core_id: CoreId| {
            SqliteEntries::query(&tx, core_id, low, high)
        })?;

        let mut entries: Vec<Entry> = sqlite_entries.into();

        limit_size(&mut entries, max_size);

        Ok(entries)
    }

    fn term(&self, idx: u64) -> RaftResult<u64> {
        let sqlite_entry: SqliteEntry = self.inside_transaction(|tx: &Transaction, core_id: CoreId| {
            SqliteEntry::query(tx, core_id, idx)
        })?;

        let entry: Entry = sqlite_entry.into();

        Ok(entry.get_term())
    }

    fn first_index(&self) -> RaftResult<u64> {
        let idx = self.inside_transaction(|tx: &Transaction, core_id: CoreId| {
            SqliteEntry::first_index(tx, core_id)
        })?;

        // Don't return first dummy entry
        Ok(idx + 1)
    }

    fn last_index(&self) -> RaftResult<u64> {
        let idx = self.inside_transaction(|tx: &Transaction, core_id: CoreId| {
            SqliteEntry::last_index(tx, core_id)
        })?;

        Ok(idx)
    }

    fn snapshot(&self) -> RaftResult<Snapshot> {
        let sqlite_snapshot: SqliteSnapshot = self.inside_transaction(|tx: &Transaction, core_id: CoreId| {
            SqliteSnapshot::query(tx, core_id)
        })?;

        let snapshot: Snapshot = sqlite_snapshot.into();

        Ok(snapshot)
    }
}

impl Debug for SqliteStorage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SqliteStorage")
            .field("id", &self.id)
            .field("debug", &self.debug_view())
            .finish()
    }
}
