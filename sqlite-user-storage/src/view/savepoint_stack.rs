use error::Result;
use rusqlite::Connection;
use rusqlite::Savepoint;
use sqlite_requests::connection::access::ReadWrite;
use sqlite_requests::connection::AccessConnection;
use sqlite_requests::request::SqliteRequest;
use std::path::Path;


// TODO: evaluate savepoint stack with debug sql statements
// sqlite_requests are currently incompatible with savepoints
// only refactor requests if savepoint stack seems valid
// evaluate request/index/entry distinction

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


    fn test(&mut self) -> Result<()> {
        let mut sp1 = self.conn.savepoint()?;

        let sp2 = sp1.savepoint()?;

        Ok(())
    }
}

pub struct SavepointStack<'conn> {
    access_conn: AccessConnection<ReadWrite>,
    savepoints: Vec<(Savepoint<'conn>, u64)>,
}

impl<'conn> SavepointStack<'conn> {
    pub fn push(&mut self, request: SqliteRequest) -> Result<()> {
        let (last_sp, index) = self.savepoints.last_mut().unwrap();

        let new_last_sp = last_sp.savepoint()?;

        // cant execute request on savepoint


        unimplemented!()
    }
}