use crate::connection::access::Access;
use crate::error::Result;
use crate::request::Request;
use rusqlite::Connection;
use rusqlite::Transaction;
use std::path::Path;

pub mod access;

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

    pub fn run<R>(&mut self, request: &R) -> Result<R::Response>
        where R: Request<A> {
        self.inside_transaction(|tx| request.apply_to_tx(tx))
    }

    pub(crate) fn inside_transaction<T>(&mut self, mut f: impl FnMut(&mut AccessTransaction<'_, A>) -> Result<T>) -> Result<T> {
        let mut access_tx = self.access_transaction()?;

        let res = f(&mut access_tx)?;

        access_tx.into_inner().commit()?;

        Ok(res)
    }

    fn access_transaction(&mut self) -> Result<AccessTransaction<'_, A>> {
        Ok(AccessTransaction {
            tx: self.conn.transaction()?,
            _access: self.access.clone(),
        })
    }
}

pub struct AccessTransaction<'conn, A: Access> {
    tx: Transaction<'conn>,
    _access: A,
}

impl<'conn, A: Access> AccessTransaction<'conn, A> {
    pub fn as_mut_inner(&mut self) -> &mut Transaction<'conn> {
        &mut self.tx
    }

    pub fn into_inner(self) -> Transaction<'conn> {
        self.tx
    }
}
