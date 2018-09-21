extern crate sqlite_raft_storage;
extern crate failure;

use sqlite_raft_storage::SqliteStorage;
use failure::Error;

fn main() -> Result<(), Error> {
    let _sqlite_storage = SqliteStorage::open("res/debug/raft_storage_debug.sqlite3")?;

    Ok(())
}
