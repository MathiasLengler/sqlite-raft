extern crate grpc_experiments;
/// Evaluate:
///
/// # Client <-> Node
/// Timeout behaviour
/// Propose a sql command -> get back response from node
///
/// ## API
///
/// `run(enum SqliteRequest) -> ?`
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
#[macro_use]
extern crate log;
extern crate sqlite_commands;
extern crate futures;

use grpc_experiments::proto::raftsqlite_grpc::RaftSqliteClientApi;
use grpcio::RpcContext;
use grpcio::UnarySink;
use sqlite_commands::connection::AccessConnection;
use sqlite_commands::connection::ReadOnly;
use sqlite_commands::connection::ReadWrite;
use sqlite_commands::proto::*;
use sqlite_commands::query::Query;
use std::sync::Arc;
use std::sync::Mutex;
use futures::Future;

#[derive(Clone)]
struct RaftSqliteClientAPIService {
    read_write_conn: Arc<Mutex<AccessConnection<ReadWrite>>>,
    read_only_conn: Arc<Mutex<AccessConnection<ReadOnly>>>,
}

impl RaftSqliteClientApi for RaftSqliteClientAPIService {
    fn query(&self, ctx: RpcContext, req: ProtoQueryRequest, sink: UnarySink<ProtoQueryResponse>) {
        let query: Query = req.into();

        eprintln!("query = {:?}", query);


        let query_result = {
            let mut conn = self.read_only_conn.lock().unwrap();

            // TODO: error handling: modify ProtoQueryResponse? Has UnarySink built in error passing?
            conn.run(&query).unwrap()
        };

        let f = sink.success(query_result.into())
            .map_err(move |e| error!("failed to reply {:?}: {:?}", query, e));

        ctx.spawn(f)
    }

    fn execute(&self, ctx: RpcContext, req: ProtoExecuteRequest, sink: UnarySink<ProtoExecuteResponse>) {
        unimplemented!()
    }

    fn bulk_query(&self, ctx: RpcContext, req: ProtoBulkQueryRequest, sink: UnarySink<ProtoBulkQueryResponse>) {
        unimplemented!()
    }

    fn bulk_execute(&self, ctx: RpcContext, req: ProtoBulkExecuteRequest, sink: UnarySink<ProtoBulkExecuteResponse>) {
        unimplemented!()
    }
}

fn main() {
    println!("Hello, world!");
}
