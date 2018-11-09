use connection::access::Access;
use error::Result;
use request::Request;
use rusqlite::Connection;
use std::path::Path;
use std::ops::Deref;

pub mod access;

pub struct AccessConnection<A: Access> {
    conn: Connection,
    access: A,
}

impl<A: Access> AccessConnection<A> {
    pub fn new(access: A, conn: Connection) -> AccessConnection<A> {
        AccessConnection {
            conn,
            access,
        }
    }

    pub fn open<P: AsRef<Path>>(access: A, path: P) -> Result<AccessConnection<A>> {
        let conn = access.open(path)?;

        Ok(AccessConnection {
            conn,
            access,
        })
    }

    pub fn run<R>(&mut self, request: &R) -> Result<R::Response>
        where R: Request<A> {
        self.inside_transaction(|tx| request.apply_to_conn(tx))
    }

    pub(crate) fn inside_transaction<T>(&mut self, mut f: impl FnMut(&mut AccessConnectionRef<A>) -> Result<T>) -> Result<T> {
        let sp = self.conn.savepoint()?;

        let res = {
            let mut access_tx = AccessConnectionRef {
                conn_ref: sp.deref(),
                _access: self.access.clone(),
            };

            f(&mut access_tx)?
        };

        sp.commit()?;

        Ok(res)
    }
}

pub struct AccessConnectionRef<'conn, A: Access> {
    conn_ref: &'conn Connection,
    _access: A,
}

impl<'conn, A: Access> AccessConnectionRef<'conn, A> {
    pub fn new(conn_ref: &'conn Connection, access: A) -> AccessConnectionRef<'conn, A> {
        AccessConnectionRef {
            conn_ref,
            _access: access,
        }
    }
}

impl<'conn, A: Access> Deref for AccessConnectionRef<'conn, A> {
    type Target = Connection;

    fn deref(&self) -> &Connection {
        self.conn_ref
    }
}
