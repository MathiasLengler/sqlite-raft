use error::Result;
use rusqlite::Connection;
use rusqlite::Savepoint;
use sqlite_requests::connection::access::ReadWrite;
use sqlite_requests::connection::AccessSavepoint;
use sqlite_requests::request::Request;
use sqlite_requests::request::SqliteRequest;

/// # Rollback
/// SAVEPOINT 0
/// #1 EXECUTE
/// SAVEPOINT 1
/// #2 EXECUTE
/// SAVEPOINT 2
/// #3 EXECUTE
/// ROLLBACK TO 1
/// DB State after #1 Execute
///
/// # Release
/// SAVEPOINT 0
/// #1 EXECUTE
/// SAVEPOINT 1
/// #2 EXECUTE
/// SAVEPOINT 2
/// #3 EXECUTE
/// RELEASE 1
///
/// New state:
/// SAVEPOINT 0
/// #1 EXECUTE
/// SAVEPOINT 1
/// #2 EXECUTE
/// #3 EXECUTE
pub struct NestedSavepoint<'conn> {
    conn: &'conn mut Connection,
    depth: u64,
}

impl<'conn> NestedSavepoint<'conn> {
    fn new(conn: &mut Connection) -> NestedSavepoint {
        NestedSavepoint {
            conn,
            depth: 0,
        }
    }
    fn sql_savepoint(name: &str) -> String {
        format!("SAVEPOINT {}", name)
    }
    fn sql_release(name: &str) -> String {
        format!("RELEASE {}", name)
    }
    fn sql_rollback(name: &str) -> String {
        format!("ROLLBACK TO {}", name)
    }

    pub fn push(&mut self, request: SqliteRequest) -> Result<()> {
        // TODO: create own savepoint
        // TODO: clean up savepoint on error
        // TODO: separate request from nested savepoint

        let mut sp = self.conn.savepoint()?;

        let mut access_sp = AccessSavepoint::new(sp, ReadWrite);

        let response = request.apply_to_sp(&mut access_sp)?;

        let sp = access_sp.into_inner();

        sp.commit()?;

        self.depth += 1;

        Ok(())
    }

    // TODO:
    pub fn rollback_to(&mut self, depth: u64) -> Result<()> {
        unimplemented!()
    }
}

