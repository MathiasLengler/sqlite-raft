use rusqlite::Connection;

use error::Result;
use sqlite_requests::connection::access::ReadWrite;
use sqlite_requests::connection::AccessConnectionRef;
use sqlite_requests::request::Request;
use sqlite_requests::request::SqliteRequest;
use sqlite_requests::request::SqliteResponse;

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
#[derive(Debug)]
pub struct NestedSavepoint<'conn> {
    conn: &'conn Connection,
    depth: u64,
}

// TODO: API for: commit/rollback everything, drop guard
impl<'conn> NestedSavepoint<'conn> {
    pub fn new(conn: &Connection) -> NestedSavepoint {
        NestedSavepoint {
            conn,
            depth: 0,
        }
    }
    fn sql_savepoint(depth: u64) -> String {
        format!("SAVEPOINT {}", Self::savepoint_name(depth))
    }
    // TODO: when is this used?
    fn sql_release(depth: u64) -> String {
        format!("RELEASE {}", Self::savepoint_name(depth))
    }
    fn sql_rollback(depth: u64) -> String {
        format!("ROLLBACK TO {}", Self::savepoint_name(depth))
    }
    fn savepoint_name(depth: u64) -> String {
        format!("_nested_sp_{}", depth)
    }

    fn savepoint(&mut self) -> Result<()> {
        self.conn.execute_batch(&Self::sql_savepoint(self.depth))?;

        self.depth += 1;

        Ok(())
    }

    pub fn push(&mut self, request: &SqliteRequest) -> Result<SqliteResponse> {
        // TODO: separate request from nested savepoint (?)

        self.savepoint()?;

        match self.apply_request(request) {
            Ok(response) => Ok(response),
            Err(err) => {
                self.pop()?;

                Err(err)
            }
        }
    }

    fn apply_request(&self, request: &SqliteRequest) -> Result<SqliteResponse> {
        let mut access_conn = AccessConnectionRef::new(&self.conn, ReadWrite);

        let response = request.apply_to_conn(&mut access_conn)?;

        Ok(response)
    }

    pub fn pop(&mut self) -> Result<()> {
        ensure!(self.depth > 0, "No nested savepoint to pop.");

        let old_depth = self.depth - 1;

        self.rollback_to(old_depth)
    }

    pub fn rollback_to(&mut self, depth: u64) -> Result<()> {
        ensure!(self.depth > depth, "Rollback target depth must be less than current depth.");

        self.conn.execute_batch(&Self::sql_rollback(depth))?;
        self.conn.execute_batch(&Self::sql_release(depth))?;

        self.depth = depth;

        Ok(())
    }
}

