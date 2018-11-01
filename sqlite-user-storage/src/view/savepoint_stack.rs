use error::Result;
use rusqlite::Connection;
use rusqlite::Savepoint;
use sqlite_requests::connection::access::ReadWrite;
use sqlite_requests::connection::AccessConnection;
use sqlite_requests::connection::AccessSavepoint;
use sqlite_requests::request::Request;
use sqlite_requests::request::SqliteRequest;
use sqlite_requests::request::SqliteResponse;
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
    // TODO: does not seem possible:
    // to create a new savepoint, the old one is borrowed for the entire livetime of the new savepoint.
    // As a vec gives us mut access of all elements, this seems to be a contradiction.
    // TODO: evaluate own nested savepoint implementation using conn.execute_batch() (complexity?)
    pub fn push(&'conn mut self, request: SqliteRequest) -> Result<()> {
        let (new_last_sp, last_index) = {
            let (last_sp, last_index) = self.savepoints.last_mut().unwrap();

            let new_last_sp = last_sp.savepoint()?;

            (new_last_sp, last_index.clone())
        };

//        let mut access_sp = AccessSavepoint::new(new_last_sp, ReadWrite);
//
//        let response = request.apply_to_sp(&mut access_sp)?;
//
//        let new_last_sp = access_sp.into_inner();

        self.savepoints.push((new_last_sp, 1 + last_index.clone()));

        Ok(())
    }
}