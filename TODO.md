# TODO

## Rust CLI client
- PoC use case of DB
- Potential Features
  - Map/Reduce query
    - quite complex
    - makes use of replication
  - Talk to multiple nodes
    - Avoid redirected traffic when proposing to a follower

## Remote database API
- Transport
  - bincode over TCP
  - HTTP server
    - JSON API
      - rqlite template
    - Protobuf
  - RPC (how to support raft lib messages?)
    - tarpc
    - gRPC
- Features
  - Runs on every node (no single point of failure)
  - Multiple concurrent clients
  - Query leader/follower (OLTP/OLAP)
    - Redirect to leader?
  - Return result set
    - Statement::query_map
      - f: (&Row) -> Vec< rusqlite::types::Value >
        - Vec length == Row::column_count
      - MappedRows iterator returns Vec< Value >
    - Transpose result set? Type Affinity reliable?
- SQLite command data structure
  - Different SQL statement types
    - Single SQL statement
    - Prepared statements (Batching)
    - Series of different statements
  - Ensure mutability
    - Connection::open_with_flags
      - READ_ONLY
      - READ_WRITE
    - Data Manipulation Language (DML)
      - Select (no side effects)
        - Not in Raft log
        - Could also be in the log for read consistency
      - Insert / Update / Delete (side effects)
    - Data Definition Language (DDL)
      - Create / Alter / Drop (side effects)
    - Nested statements?
  - Ensure deterministic execution
    - replace random()
    - etc

## Storage
- SQLite storage trait implementation
  - Init/load DB
  - Compaction implementation
    - Online SQLite backup API
    - In Raft Log as BLOB?
  - Two separate DBs (Multi-file Commit)
    - User DB
    - Raft Log
      - Table Log
      - Table Courser (hard/soft state?)
      - next_propose_id
  - Test against MemStorage
    - reuse existing tests?
  - Snapshot of DB at specific log index
    - Pro
      - deterministic query possible
    - Con
      - Storage requirements
      - Can't roll back applied statements
- Node `WriteStorage` trait
  - template MemStorageCore
  - node should be agnostic which storage type is used

## Node communication trait
- Implementations
  - Channel
  - TCP
- Node only has knowledge of the trait

## Node launch API
- Communication type
  - Channel cluster
    - Mesh (Implemented)
    - Message Bus
      - allows centralized interception of messages (hooks)
  - TCP node
    - Parameters
      - Peer ids?
      - Peer urls
- Storage Type
  - MemStorage
  - SqliteStorage
- Docker TCP cluster

## Testing hooks
- Cluster
  - Single step whole cluster
  - Get handle of node
  - Support channel communication
- Node
  - Availability
    - stop / resume / restart
    - Intercept before/while specific `Ready` processing stages
  - force time out (election)
  - new request/propose
  - Manual step
  - Inject custom randomness?
  - State machine application result callback
- Communication
  - Disable specific channel
  - Isolate single Node
  - Segment Nodes
  - Reorder Messages
  - Simulate Latency
  - Drop single message
  - Set packet loss rate
    - every n'th
    - random

## Dashboard
- WS reporting API
  - Node states
  - In flight messages
- Integrate testing hooks

## Raft
- Dynamic cluster
  - TODO: handle EntryConfChange
- Abstract raft stepping/ready processing logic
  - Could be useful for testing hooks

## Unresolved Questions
- SQL Transactions / Multiple Statements / Intermediate results
- SQL Statement Validation before state machine application
  - Fundamental problem: race condition with other proposed statements
    - eg ALTER TABLE followed by INSERT against old schema
    - Append log but don't apply to state machine
      - mark bad log entries (skip on startup/restore)
  - Try and abort Transaction / Savepoint
    - Useful for sanity/syntax check
    - Fast fail of request
      - Does prepared statement validate before executing? (alternative)
- SQL user defined functions
  - Usecase?
- When/how to compact in raft-rs
  - MemStorageCore template