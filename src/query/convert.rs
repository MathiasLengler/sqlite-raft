use query::Query;
use proto::ProtoQueryRequest;

impl From<Query> for ProtoQueryRequest {
    fn from(query: Query) -> Self {
        let mut proto_query_request = ProtoQueryRequest::new();
        proto_query_request.set_sql(query.sql);
        proto_query_request.set_queued_parameters(query.queued_parameters.into());
        proto_query_request
    }
}

impl From<ProtoQueryRequest> for Query {
    fn from(mut proto_query_request: ProtoQueryRequest) -> Self {
        Query {
            sql: proto_query_request.take_sql(),
            queued_parameters: proto_query_request.take_queued_parameters().into(),
        }
    }
}

// TODO: ProtoQueryResponse
