use error::Result;
use model::core::CoreTx;
use model::entry::SqliteEntries;
use model::entry::SqliteEntry;
use SqliteStorage;

#[derive(Debug)]
pub struct SqliteStorageDebugView {
    query_all: Result<SqliteEntries>,
    first_index: Result<u64>,
    last_index: Result<u64>,
}

impl<'a> From<&'a SqliteStorage> for SqliteStorageDebugView {
    fn from(sqlite_storage: &SqliteStorage) -> Self {
        sqlite_storage.inside_transaction(|core_tx: &CoreTx| {
            Ok(Self {
                query_all: SqliteEntries::query_all(core_tx),
                first_index: SqliteEntry::first_index(core_tx),
                last_index: SqliteEntry::last_index(core_tx),
            })
        }).unwrap()
    }
}

