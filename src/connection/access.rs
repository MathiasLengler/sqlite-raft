use std::path::Path;
use rusqlite::Connection;
use error::Result;
use rusqlite::OpenFlags;

/// Implemented by different access levels used by `AccessConnection`.
pub trait Access: Copy {
    /// Open a rusqlite connection using this access level.
    fn open<P: AsRef<Path>>(&self, path: P) -> Result<Connection>;
}

/// A marker trait implemented by access levels which provide read access to the DB.
///
/// Requests can constrain the access level using this trait.
pub trait ReadAccess: Access {}

/// Implemented by access levels which provide write access to the DB.
///
/// Requests can constrain the access level using this trait.
pub trait WriteAccess: Access {}

/// Provide only read access.
///
/// Can be used to run: `Query`, `BulkQuery`, `SqliteQuery`.
#[derive(Debug, Copy, Clone)]
pub struct ReadOnly;

impl Access for ReadOnly {
    fn open<P: AsRef<Path>>(&self, path: P) -> Result<Connection> {
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY,
        )?;

        Ok(conn)
    }
}

impl ReadAccess for ReadOnly {}

/// Provide only write access.
///
/// Can be used to run: `Execute`, `BulkExecute`, `SqliteExecute`.
#[derive(Debug, Copy, Clone)]
pub struct WriteOnly;

impl Access for WriteOnly {
    fn open<P: AsRef<Path>>(&self, path: P) -> Result<Connection> {
        conn_open_with_read_write(path)
    }
}

impl WriteAccess for WriteOnly {}

/// Provide read and write access.
///
/// Can be used to run every request.
///
/// Notably, `SqliteRequest` can only be run with this access level.
#[derive(Debug, Copy, Clone)]
pub struct ReadWrite;

impl Access for ReadWrite {
    fn open<P: AsRef<Path>>(&self, path: P) -> Result<Connection> {
        conn_open_with_read_write(path)
    }
}

impl ReadAccess for ReadWrite {}

impl WriteAccess for ReadWrite {}

fn conn_open_with_read_write<P: AsRef<Path>>(path: P) -> Result<Connection> {
    let conn = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
    )?;

    Ok(conn)
}