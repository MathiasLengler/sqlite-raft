extern crate failure;
extern crate sqlite_raft_storage;

use failure::Error;
use sqlite_raft_storage::SqliteStorage;

fn main() -> Result<(), Error> {
    let _sqlite_storage = SqliteStorage::open("res/debug/raft_storage_debug.sqlite3")?;

    Ok(())
}
