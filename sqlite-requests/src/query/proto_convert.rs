use proto::ProtoQueryRequest;
use proto::ProtoQueryResponse;
use proto::ProtoQueryResultSet;
use proto::ProtoQueryResultRow;
use proto::ProtoBulkQueryRequest;
use proto::ProtoBulkQueryResponse;
use proto::ProtoValue;
use query::Query;
use query::QueryResultSet;
use query::QueryResultRow;
use query::BulkQuery;

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

impl From<Vec<QueryResultSet>> for ProtoQueryResponse {
    fn from(query_results: Vec<QueryResultSet>) -> Self {
        let mut proto_query_response = ProtoQueryResponse::new();
        let proto_query_result_set: Vec<ProtoQueryResultSet> =
            query_results.into_iter().map(Into::into).collect();
        proto_query_response.set_query_result_sets(proto_query_result_set.into());
        proto_query_response
    }
}

impl From<ProtoQueryResponse> for Vec<QueryResultSet> {
    fn from(mut proto_query_response: ProtoQueryResponse) -> Self {
        proto_query_response
            .take_query_result_sets()
            .into_vec()
            .into_iter()
            .map(Into::into)
            .collect()
    }
}

impl From<QueryResultSet> for ProtoQueryResultSet {
    fn from(query_result: QueryResultSet) -> Self {
        let mut proto_query_result = ProtoQueryResultSet::new();
        let vec_proto_query_result_row: Vec<ProtoQueryResultRow> =
            query_result.rows.into_iter().map(Into::into).collect();
        proto_query_result.set_rows(vec_proto_query_result_row.into());
        proto_query_result
    }
}

impl From<ProtoQueryResultSet> for QueryResultSet {
    fn from(mut proto_query_result: ProtoQueryResultSet) -> Self {
        QueryResultSet {
            rows: proto_query_result
                .take_rows()
                .into_vec()
                .into_iter()
                .map(Into::into)
                .collect()
        }
    }
}

impl From<QueryResultRow> for ProtoQueryResultRow {
    fn from(query_result_row: QueryResultRow) -> Self {
        let mut proto_query_result_row = ProtoQueryResultRow::new();
        let vec_proto_value: Vec<ProtoValue> =
            query_result_row.row.into_iter().map(Into::into).collect();
        proto_query_result_row.set_row(vec_proto_value.into());
        proto_query_result_row
    }
}

impl From<ProtoQueryResultRow> for QueryResultRow {
    fn from(mut proto_query_result_row: ProtoQueryResultRow) -> Self {
        QueryResultRow {
            row: proto_query_result_row
                .take_row()
                .into_vec()
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

impl From<BulkQuery> for ProtoBulkQueryRequest {
    fn from(bulk_query: BulkQuery) -> Self {
        let mut proto_bulk_query_request = ProtoBulkQueryRequest::new();
        let vec_proto_query_request: Vec<ProtoQueryRequest> =
            bulk_query.queries.into_iter().map(Into::into).collect();
        proto_bulk_query_request.set_queries(vec_proto_query_request.into());
        proto_bulk_query_request
    }
}

impl From<ProtoBulkQueryRequest> for BulkQuery {
    fn from(mut proto_bulk_query_request: ProtoBulkQueryRequest) -> Self {
        BulkQuery {
            queries: proto_bulk_query_request
                .take_queries()
                .into_vec()
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

impl From<Vec<Vec<QueryResultSet>>> for ProtoBulkQueryResponse {
    fn from(vec_vec_query_response: Vec<Vec<QueryResultSet>>) -> Self {
        let mut proto_bulk_query_response = ProtoBulkQueryResponse::new();
        let vec_proto_query_response: Vec<ProtoQueryResponse> =
            vec_vec_query_response.into_iter().map(Into::into).collect();
        proto_bulk_query_response.set_query_responses(vec_proto_query_response.into());
        proto_bulk_query_response
    }
}

impl From<ProtoBulkQueryResponse> for Vec<Vec<QueryResultSet>> {
    fn from(mut proto_bulk_query_response: ProtoBulkQueryResponse) -> Self {
        proto_bulk_query_response
            .take_query_responses()
            .into_vec()
            .into_iter()
            .map(Into::into)
            .collect()
    }
}

