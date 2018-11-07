use error::Result;
use rusqlite::Connection;
use rusqlite::Savepoint;
use sqlite_requests::connection::access::ReadWrite;
use sqlite_requests::connection::AccessConnection;
use sqlite_requests::connection::AccessSavepoint;
use sqlite_requests::request::Request;
use sqlite_requests::request::SqliteRequest;
use sqlite_requests::request::SqliteResponse;
use std::cell::RefCell;
use std::cell::RefMut;
use std::path::Path;


// TODO: evaluate request/index/entry distinction
// TODO: savepoint stack could be used for speculative execution of requests.
// cache response, wait for committed entry from leader, return response
// only consistent if the execution had the same predecessor entry as the committed entry (?)


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
/// - Query a specific index?
///     - roll back to the index
///         - can't work on other request in parallel
///     - another instance
///         - current view
///             - does not need to do the snapshot stack trickery
///             - enough for PoC
///         - specific index view
///             - snapshot stack to allow for forward and backwards modifications
/// - Is there a requirement for a seed process after restart?
///     - compare with committed entry behaviour and persistence in raft step logic
///     - application of committed entries is async (raft logic can't block) =>
///         - seed process required (?)
///     - on startup:
///         - raft sends all committed entries
///             - slow
///         - raft sends missing entries on demand
///             - bidirectional request flow (would be unidirectional otherwise)
///     - the specific index view does not actively follow the committed entries
///         - read only view of entries would be convenient for implementation (required?)
/// - Wrapper of SQL Request + Index seems to be beneficial
///     - validation
///     - do not execute duplicate requests
///     - same for read only view of fixed index view
/// - Read only API parallel to Raft seems to be needed
/// - Halt the world in event of snapshot?
///     - could define "toxic" index for specific view
///     - current view needs to be advanced far enough before snapshot gets applied
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
    // to create a new savepoint, the old one is borrowed for the entire lifetime of the new savepoint.
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
