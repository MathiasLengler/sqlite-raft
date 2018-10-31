use connection::access::Access;
use error::Result;
use request::Request;
use rusqlite::Connection;
use rusqlite::Savepoint;
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
        self.inside_transaction(|tx| request.apply_to_sp(tx))
    }

    pub(crate) fn inside_transaction<T>(&mut self, mut f: impl FnMut(&mut AccessSavepoint<A>) -> Result<T>) -> Result<T> {
        let mut access_tx = self.access_savepoint()?;

        let res = f(&mut access_tx)?;

        access_tx.into_inner().commit()?;

        Ok(res)
    }

    fn access_savepoint(&mut self) -> Result<AccessSavepoint<A>> {
        Ok(AccessSavepoint {
            sp: self.conn.savepoint()?,
            _access: self.access.clone(),
        })
    }
}

pub struct AccessSavepoint<'conn, A: Access> {
    sp: Savepoint<'conn>,
    _access: A,
}

impl<'conn, A: Access> AccessSavepoint<'conn, A> {
    pub fn new(sp: Savepoint<'conn>, access: A) -> AccessSavepoint<'conn, A> {
        AccessSavepoint {
            sp,
            _access: access,
        }
    }

    pub fn as_mut_inner(&mut self) -> &mut Savepoint<'conn> {
        &mut self.sp
    }

    pub fn into_inner(self) -> Savepoint<'conn> {
        self.sp
    }
}
