/// Evaluate:
///
/// # Client <-> Node
/// Timeout behaviour
/// Propose a sql command -> get back response from node
///
/// # Node <-> Node
/// Pass raft-rs messages
/// Bidirectional
/// Two send endpoints
///
/// ## Testing
/// Communication Trait
/// `send(Msg) -> ()`
///
/// # sqlite-commands
/// - port data structures to protobuf
///     - How to add functionality to generated structs?
/// - transpile data structures to profobuf
///     - serde?
/// - Custom wrapper around serde bincode
///     - use only transport infrastructure
///

fn main() {
    println!("Hello, world!");
}
