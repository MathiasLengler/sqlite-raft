#[macro_use]
extern crate failure;
extern crate protobuf;
extern crate raft;
extern crate rusqlite;

use error::Result;
use model::core::CoreId;
use model::entry::SqliteEntries;
use model::hard_state::SqliteHardState;
use model::snapshot::SqliteSnapshot;
use raft::eraftpb::Entry;
use raft::eraftpb::Snapshot;
use raft::RaftState;
use raft::Result as RaftResult;
use raft::Storage;
use rusqlite::Connection;
use rusqlite::Transaction;
use std::path::Path;
use std::sync::RwLock;

mod model;
pub mod error;

// TODO: Test against MemStorageCore
// TODO: use in sqlite-raft
// TODO: TryFrom for "as" casts and proto conversions (take_)

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
        let mut storage = SqliteStorage {
            conn: RwLock::new(Connection::open(path)?),
            id: 0.into(),
        };

        storage.init()?;

        Ok(storage)
    }

    fn init(&self) -> Result<()> {
        self.inside_transaction(|mut tx: &mut Transaction, core_id: CoreId| {
            tx.execute_batch(SqliteStorage::SQL_ON_OPEN)?;

            SqliteStorage::create_tables_if_not_exists(&mut tx)?;

            if !core_id.exists(&mut tx)? {
                core_id.insert(&mut tx)?;

                SqliteHardState::default().insert_or_replace(&mut tx, core_id)?;
                SqliteSnapshot::default().insert_or_replace(&mut tx, core_id)?;
                SqliteEntries::default().insert_or_replace(&mut tx, core_id)?;
            }

            Ok(())
        })
    }

    fn create_tables_if_not_exists(tx: &mut Transaction) -> Result<()> {
        let mut stmt = tx.prepare(SqliteStorage::SQL_EXISTS)?;
        if !stmt.exists(&[])? {
            tx.execute_batch(SqliteStorage::SQL_CREATE_TABLES)?;
        }
        Ok(())
    }

    fn inside_transaction<T>(&self, mut f: impl FnMut(&mut Transaction, CoreId) -> Result<T>) -> Result<T> {
        // TODO: handle poisoned lock
        let mut wl_conn = self.conn.write().unwrap();

        let mut tx = wl_conn.transaction()?;

        let res = f(&mut tx, self.id)?;

        tx.commit()?;

        Ok(res)
    }
}

impl Storage for SqliteStorage {
    fn initial_state(&self) -> RaftResult<RaftState> {
        unimplemented!()
    }

    fn entries(&self, low: u64, high: u64, max_size: u64) -> RaftResult<Vec<Entry>> {
        use raft::util::limit_size;

        let sqlite_entries: SqliteEntries = self.inside_transaction(|mut tx: &mut Transaction, core_id: CoreId| {
            SqliteEntries::query(&mut tx, core_id, low, high)
        })?;

        let mut entries: Vec<Entry> = sqlite_entries.into();

        limit_size(&mut entries, max_size);

        Ok(entries)
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