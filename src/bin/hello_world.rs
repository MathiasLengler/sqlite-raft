/// Evaluate:
///
/// # Client <-> Node
/// Timeout behaviour
/// Propose a sql command -> get back response from node
///
/// ## API
///
/// `run(enum SqliteCommand) -> ?`
///
/// vs
///
/// Separate methods for each command type:
/// - Query
/// - Execute
/// - BulkQuery
/// - BulkExecute
///
/// # Node <-> Node
/// Pass raft-rs messages
/// Bidirectional / Two send endpoints
///
/// ## Testing
/// Communication Trait (for channel transport)
/// `send(Msg) -> ()` (async)
///
/// # Node
///
/// client request -> raft propose -> wait for raft commit -> find callback -> complete client request
///
/// Must bridge Future/Raft stepping API
///
/// # sqlite-commands
/// - port data structures to protobuf
///     - How to add functionality to generated structs?
///     - sqlite-commands already implemented in rust
/// - Custom wrapper around serde payload
///     - use gRPC transport infrastructure (timeout, compression, ...)
///     - NodeToNode as bincode, ClientToNode as JSON
///     - mixed client experience
///         - has to generate JSON and gRPC
///         - plain JSON better? HTTP Server?
///         - bad for bandwidth/performance
/// - Hybrid
///     - port only the API layer for Clients
///         - big percentage of sqlite-commands
///     - mirror API and provide from impls

extern crate grpcio;
extern crate grpc_experiments;

use grpc_experiments::proto_gen::helloworld::QueryRequest;
use grpc_experiments::proto_gen::helloworld::SerdeContainer_oneof_format;

fn main() {
    let request = QueryRequest::new();

    match request.get_payload().format {
        Some(SerdeContainer_oneof_format::json(json)) => {},
        Some(SerdeContainer_oneof_format::rust_bincode(json)) => {},
        None => {},
    }


    println!("Hello, world!");


}
