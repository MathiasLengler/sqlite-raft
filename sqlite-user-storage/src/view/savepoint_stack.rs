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


// TODO: evaluate request/index/entry distinction
// TODO: savepoint stack could be used for speculative execution of requests.
// cache response, wait for committed entry from leader, return response
// only consistent if state of execution the same of predecessor of commited entry (?)


/// TODO: Fundamental question:
/// How does user storage get access to entries/sequence of requests?
/// # Actor/Channel with new requests
/// - Asynchronous (does not block the main raft state machine)
/// - Keeps a up to date view of the committed entries
/// - does need a private/hidden table in the user DB to persist the current applied index of the DB
///     - otherwise there could be dropped requests (inconsistent/nondeterministic)
///     - could be a third "user db state db"
///         - nice to have
///         - only the user db/view thread writes and reads from it
///         - rest of coordination goes through channels
/// - Black Box behaviour must be deterministic (its the Raft state machine)
///
///
/// # Attached DB
/// - incompatible with savepoint stack for rollback:
///     - savepoints/transactions lock all attached DBs.
///     - A locked DB cannot be detached.
///     - Cannot read new entries from attached raft-DB while keeping savepoint stack for rollback (cant see updates).


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
}

pub struct SavepointStack<'conn> {
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

///
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
    conn: &'conn Connection,

}

