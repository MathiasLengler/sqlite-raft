#[macro_use]
extern crate failure;
extern crate protobuf;
extern crate raft;
extern crate rusqlite;

use error::Result;
use model::core::CoreId;
use model::hard_state::SqliteHardState;
use raft::eraftpb::Entry;
use raft::eraftpb::Snapshot;
use raft::RaftState;
use raft::Result as RaftResult;
use raft::Storage;
use rusqlite::Connection;
use rusqlite::Transaction;
use std::path::Path;
use model::snapshot::SqliteSnapshot;

mod model;
pub mod error;

// TODO: Test against MemStorageCore
// TODO: use in sqlite-raft
// TODO: TryFrom for "as" casts and proto conversions (take_)

pub struct SqliteStorage {
    conn: Connection,
    id: CoreId,
}

impl SqliteStorage {
    const SQL_ON_OPEN: &'static str =
        include_str!("../res/sql/on_open.sql");
    const SQL_EXISTS: &'static str =
        include_str!("../res/sql/exists.sql");
    const SQL_INIT: &'static str =
        include_str!("../res/sql/init.sql");

    pub fn open<P: AsRef<Path>>(path: P) -> Result<SqliteStorage> {
        let mut storage = SqliteStorage {
            conn: Connection::open(path)?,
            id: 0.into(),
        };

        storage.init_if_not_exists()?;

        Ok(storage)
    }

    fn init_if_not_exists(&mut self) -> Result<()> {
        let mut tx = self.conn.transaction()?;

        tx.execute_batch(SqliteStorage::SQL_ON_OPEN)?;

        SqliteStorage::create_tables_if_not_exists(&mut tx)?;

        if !self.id.exists(&mut tx)? {
            self.id.insert(&mut tx)?;

            SqliteHardState::default().insert_or_replace(&mut tx, self.id)?;
            SqliteSnapshot::default().insert_or_replace(&mut tx, self.id)?;

            // TODO: init entries
        }

        tx.commit()?;

        Ok(())
    }

    fn create_tables_if_not_exists(tx: &mut Transaction) -> Result<()> {
        let mut stmt = tx.prepare(SqliteStorage::SQL_EXISTS)?;
        if !stmt.exists(&[])? {
            tx.execute_batch(SqliteStorage::SQL_INIT)?;
        }
        Ok(())
    }
}

impl Storage for SqliteStorage {
    fn initial_state(&self) -> RaftResult<RaftState> {
        unimplemented!()
    }

    fn entries(&self, low: u64, high: u64, max_size: u64) -> RaftResult<Vec<Entry>> {
        unimplemented!()
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