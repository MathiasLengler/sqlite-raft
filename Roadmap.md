# Roadmap

## Rust CLI client
- PoC use case of DB
- Potential Features
  - Map/Reduce query
    - quite complex
    - makes use of replication
    - At specific commit index
  - Talk to multiple nodes
    - Avoid redirected traffic when proposing to a follower

## Remote database API
- Transport
  - gRPC
    - rqlite template
- Features
  - Runs on every node (no single point of failure)
  - Multiple concurrent clients
  - Query leader/follower (OLTP/OLAP)
    - Redirect to leader?
    - Should be a setting (consistency settings rqlite)
    - PoC: consistent (no special case for query)
      - Also "Optimization of query request flow"
  - Return result set
- SQLite command data structure
  - Ensure deterministic execution
    - replace random()
    - etc
- Testing
  - Same API for cluster and a single node for A/B testing

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
- Synchronization
  - raft stepping <=> applying (expensive) sql command
    - can/should the raft stepping logic be blocked by db operations?
      - if the leader is unresponsive for too long, followers will hold an election
  - Two stage commit
    - Raft stepping thread persists storage in Raft log DB
    - Worker thread applies newly committed commands to user DB
    - Raft DB must contain two committed index pointers
      - current raft index 
      - current state of the potential trailing user DB
    - Worker thread uses multi-file commit
    - Does that work in the context of fetching/querying snapshots at specific log index?
  
## Node communication trait
- Implementations
  - Channel
  - grpc
- Node only has knowledge of the trait

## Node launch API
- Communication type
  - Channel cluster
    - Mesh (Implemented)
    - Message Bus
      - allows centralized interception of messages (hooks)
  - grpc node
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
- SQL Statement Validation before state machine application
  - Fundamental problem: race condition with other proposed statements
    - eg ALTER TABLE followed by INSERT against old schema
    - Append log but don't apply to state machine
      - mark bad log entries (skip on startup/restore)
  - Try and abort Transaction / Savepoint
    - Useful for sanity/syntax check
    - Fast fail of request
      - Does prepared statement validate before executing? (alternative)
    - Could be slow
  - Parse SQL syntax 
- When/how to compact in raft-rs
  - MemStorageCore template / Node `WriteStorage` trait

## Other Ideas

### Query a specific state
- user db on disk is the last snapshot
- only reference to snapshot in raft db
- on startup apply all committed log entries inside a transaction and keep it open
  - slow with long log
- can a separate connection rebuild the state of db to a specific log index while the other transaction is open?
  - even if there has been write access?
  - this would allow deterministic queries against a known time stamp
  - allows views of the db at any timestamp between the last snapshot and the most recently committed log entry
- alternative: keep multiple user dbs
  - committed user db
  - snapshot user db
  
### Optimization of query request flow
Client requests query to follower ->
Follower proposes append query to leader ->
Leader appends to own log ->
Leader replicates to majority of followers ->
Leader commits entry to log. Does not have to apply the query to state machine, e.g. executing the query, because he has no need for the QueryResult ->
Leader replicates committed log entry to followers ->
Follower which received client request sees, that his proposed query has been committed in the log ->
Follower executes query against the DB at the committed index ->
Returns QueryResult to client

### Threading and synchronization setup
- raft thread
  - raft stepping loop executes raft storage modifications
  - new committed_entries: sends to/notifies user-db thread
  - only transacts on raft_storage_db
- user-db thread
  - waits for new committed entries
  - multi file transaction against raft and user db
  - executes requests against user-db
    - every execute
    - only queries which have a pending request
  - writes current user db index into raft db
  - sends result to grpc thread
- grpc thread
  - passes requests/raft messages to raft thread
  - saves pending requests
  - gets result for request from user-db thread
  - responds to client
    - must be compatible with async networking

#### Snapshot creation
- not needed for MVP
- snapshot index is always less than committed entries
- Constraints:
  - all requests less than snapshot index must have been executed
  - all entries less than snapshot must have been applied to user db
  - raft thread must be able to continue to append new entries