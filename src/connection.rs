use rusqlite::Connection;
use rusqlite::OpenFlags;
use std::path::Path;
use error::Result;

pub struct AccessConnection<A: Access> {
    conn: Connection,
    access: A,
}

impl<A: Access> AccessConnection<A> {
    pub fn open_read_only<P: AsRef<Path>>(path: P) -> Result<AccessConnection<ReadOnly>> {
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_CREATE,
        )?;

        Ok(AccessConnection {
            conn,
            access: ReadOnly,
        })
    }

    pub fn open_read_write<P: AsRef<Path>>(path: P) -> Result<AccessConnection<ReadWrite>> {
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        )?;

        Ok(AccessConnection {
            conn,
            access: ReadWrite,
        })
    }

    pub(crate) fn inner_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }
}


pub trait Access {}

pub struct ReadOnly;

impl Access for ReadOnly {}

pub struct ReadWrite;

impl Access for ReadWrite {}
