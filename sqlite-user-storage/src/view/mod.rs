use std::path::Path;
use error::Result;

use rusqlite::Connection;

pub mod sqlite_snapshot;
pub mod nested_savepoint;


pub struct View {
    conn: Connection,
}

impl View {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<View> {
        let storage = View {
            conn: Connection::open(path)?
        };

        Ok(storage)
    }
}
