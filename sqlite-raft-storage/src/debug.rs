use SqliteStorage;
use error::Result;
use model::entry::SqliteEntries;
use model::entry::SqliteEntry;
use rusqlite::Transaction;
use model::core::CoreId;

#[derive(Debug)]
pub struct SqliteStorageDebugView {
    query_all: Result<SqliteEntries>,
    first_index: Result<u64>,
    last_index: Result<u64>,
}

impl<'a> From<&'a SqliteStorage> for SqliteStorageDebugView {
    fn from(sqlite_storage: &SqliteStorage) -> Self {
        sqlite_storage.inside_transaction(|tx: &Transaction, core_id: CoreId| {
            Ok(Self {
                query_all: SqliteEntries::query_all(tx, core_id),
                first_index: SqliteEntry::first_index(tx, core_id),
                last_index: SqliteEntry::last_index(tx, core_id),
            })
        }).unwrap()
    }
}

