extern crate grpc_experiments;
/// Evaluate:
///
/// # Client <-> Node
/// Timeout behaviour
/// Propose a sql command -> get back response from node
///
/// # Node <-> Node
/// Pass raft-rs messages
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

extern crate grpcio;
#[macro_use]
extern crate log;
extern crate sqlite_requests;
extern crate futures;

use grpc_experiments::proto::raftsqlite_grpc::RaftSqliteClientApi;
use grpcio::RpcContext;
use grpcio::UnarySink;
use sqlite_requests::connection::AccessConnection;
use sqlite_requests::proto::*;
use sqlite_requests::query::Query;
use std::sync::Arc;
use std::sync::Mutex;
use futures::Future;
use sqlite_requests::connection::access::ReadWrite;

#[derive(Clone)]
struct RaftSqliteClientAPIService {
    read_write_conn: Arc<Mutex<AccessConnection<ReadWrite>>>,
}

impl RaftSqliteClientApi for RaftSqliteClientAPIService {
    fn query(&self, ctx: RpcContext, req: ProtoQueryRequest, sink: UnarySink<ProtoQueryResponse>) {
        let query: Query = req.into();

        eprintln!("query = {:?}", query);


        let query_result = {
            let mut conn = self.read_write_conn.lock().unwrap();

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
