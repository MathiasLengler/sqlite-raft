//! # Actor/Channel with new requests
//! - Asynchronous (does not block the main raft state machine)
//! - Keeps a up to date view of the committed entries
//! - does need a private/hidden table in the user DB to persist the current applied index of the DB
//!     - otherwise there could be dropped requests (inconsistent/nondeterministic)
//!     - could be a third "user db state db"
//!         - nice to have
//!         - only the user db/view thread writes and reads from it
//!         - rest of coordination goes through channels
//! - Black Box behaviour must be deterministic (its the Raft state machine)
//! - Query a specific index?
//!     - roll back to the index
//!         - can't work on other request in parallel
//!     - another instance
//!         - current view
//!             - does not need to do the snapshot stack trickery
//!             - enough for PoC
//!         - specific index view
//!             - snapshot stack to allow for forward and backwards modifications
//! - Is there a requirement for a seed process after restart?
//!     - compare with committed entry behaviour and persistence in raft step logic
//!     - application of committed entries is async (raft logic can't block) =>
//!         - seed process required (?)
//!     - on startup:
//!         - raft sends all committed entries
//!             - slow
//!         - raft sends missing entries on demand
//!             - bidirectional request flow (would be unidirectional otherwise)
//!     - the specific index view does not actively follow the committed entries
//!         - read only view of entries would be convenient for implementation (required?)
//! - Wrapper of SQL Request + Index seems to be beneficial
//!     - validation
//!     - do not execute duplicate requests
//!     - same for read only view of fixed index view
//! - Read only API parallel to Raft seems to be needed
//! - Halt the world in event of snapshot?
//!     - could define "toxic" index for specific view
//!     - current view needs to be advanced far enough before snapshot gets applied
//!
//! # Attached DB
//! - incompatible with savepoint stack for rollback:
//!     - savepoints/transactions lock all attached DBs.
//!     - A locked DB cannot be detached.
//!     - Cannot read new entries from attached raft-DB while keeping savepoint stack for rollback (cant see updates).
//!
//! TODO: Fundamental question:
//! How does user storage get access to entries/sequence of requests?
//!
//! Current sketch for solution:
//!
//! user/view thread has two connections
//! gets notified if a new committed entry has been added or a request for specific index query has come in
//! reads needed entries from raft-db
//! gets sqlite-request from entry
//! hidden view table in user db to mark current index (simpler than attached state db for each view)
//! sends sqlite-response via channel to
//!
//! TODO: What is the request architecture?
//! Propose/Simple request:
//!
//! Request for query at specific index:
//!
//! TODO: How can this be integrated into to manual raft step logic?

#[macro_use]
extern crate failure;
extern crate rusqlite;
extern crate sqlite_requests;

pub mod error;
pub mod view;

// TODO: evaluate request/index/entry distinction
// TODO: savepoint stack could be used for speculative execution of requests.
// cache response, wait for committed entry from leader, return response
// only consistent if the execution had the same predecessor entry as the committed entry (?)


