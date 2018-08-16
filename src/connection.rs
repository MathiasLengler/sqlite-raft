use error::Result;
use rusqlite::Connection;
use rusqlite::OpenFlags;
use rusqlite::Transaction;
use std::path::Path;

pub struct AccessConnection<A: Access> {
    conn: Connection,
    access: A,
}

impl<A: Access> AccessConnection<A> {
    pub fn open<P: AsRef<Path>>(access: A, path: P) -> Result<AccessConnection<A>> {
        let conn = access.open(path)?;

        Ok(AccessConnection {
            conn,
            access,
        })
    }

    pub fn run<T>(&mut self, command: T) -> T::Return
        where T: Command<Access=A> {
        command.run_on_conn(self)
    }

    pub(crate) fn inside_transaction<T>(&mut self, mut f: impl FnMut(&mut AccessTransaction<A>) -> Result<T>) -> Result<T> {
        let mut access_tx = self.access_transaction()?;

        let res = f(&mut access_tx)?;

        access_tx.into_inner().commit()?;

        Ok(res)
    }

    fn access_transaction(&mut self) -> Result<AccessTransaction<A>> {
        Ok(AccessTransaction {
            tx: self.conn.transaction()?,
            _access: self.access.clone(),
        })
    }
}

pub(crate) struct AccessTransaction<'conn, A: Access> {
    tx: Transaction<'conn>,
    _access: A,
}

impl<'conn, A: Access> AccessTransaction<'conn, A> {
    pub(crate) fn as_mut(&mut self) -> &mut Transaction<'conn> {
        &mut self.tx
    }

    fn into_inner(self) -> Transaction<'conn> {
        self.tx
    }
}


pub trait Access: Copy {
    fn open<P: AsRef<Path>>(&self, path: P) -> Result<Connection>;
}

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

#[derive(Debug, Copy, Clone)]
pub struct ReadWrite;

impl Access for ReadWrite {
    fn open<P: AsRef<Path>>(&self, path: P) -> Result<Connection> {
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        )?;

        Ok(conn)
    }
}

pub trait Command {
    type Access: Access;
    type Return;

    fn run_on_conn(&self, conn: &mut AccessConnection<Self::Access>) -> Self::Return;
}
