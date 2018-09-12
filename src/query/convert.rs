use proto::ProtoQueryRequest;
use proto::ProtoQueryResponse;
use proto::ProtoQueryResult;
use proto::ProtoQueryResultRow;
use proto::ProtoValue;
use query::Query;
use query::QueryResult;
use query::QueryResultRow;

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

impl From<Vec<QueryResult>> for ProtoQueryResponse {
    fn from(query_results: Vec<QueryResult>) -> Self {
        let mut proto_query_response = ProtoQueryResponse::new();
        let vec_proto_query_result: Vec<ProtoQueryResult> =
            query_results.into_iter().map(Into::into).collect();
        proto_query_response.set_query_results(vec_proto_query_result.into());
        proto_query_response
    }
}

impl From<ProtoQueryResponse> for Vec<QueryResult> {
    fn from(mut proto_query_response: ProtoQueryResponse) -> Self {
        proto_query_response
            .take_query_results()
            .into_vec()
            .into_iter()
            .map(Into::into)
            .collect()
    }
}

impl From<QueryResult> for ProtoQueryResult {
    fn from(query_result: QueryResult) -> Self {
        let mut proto_query_result = ProtoQueryResult::new();
        let vec_proto_query_result_row: Vec<ProtoQueryResultRow> =
            query_result.rows.into_iter().map(Into::into).collect();
        proto_query_result.set_rows(vec_proto_query_result_row.into());
        proto_query_result
    }
}

impl From<ProtoQueryResult> for QueryResult {
    fn from(mut proto_query_result: ProtoQueryResult) -> Self {
        QueryResult {
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

