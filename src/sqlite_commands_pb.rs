use proto_gen::raftsqlite::ProtoQueryRequest;
use sqlite_commands::query::Query;
use sqlite_commands::parameter::QueuedParameters;

// TODO: implement the protobuf layer inside sqlite-commands crate because of visibility.
// TODO: inject generated protobuf code inside this crate.
// TODO: implement skeleton of RaftNode and RaftSqliteClientAPI
// TODO: decide where/how to implement the "Node communication trait" for the grpc RaftNode service

impl From<Query> for ProtoQueryRequest {
    fn from(query: Query) -> Self {
        let mut proto_query_request = ProtoQueryRequest::new();
//        proto_query_request.set_sql(query.sql);
//        proto_query_request.set_queued_parameters();
        proto_query_request
    }
}

impl From<ProtoQueryRequest> for Query {
    fn from(mut query_request: ProtoQueryRequest) -> Self {
//        Query {
//            sql: query_request.take_sql(),
//            queued_parameters: QueuedParameters::new_indexed(&[&[]]).unwrap(),
//        }
        unimplemented!()
    }
}
