use proto::ProtoSqliteRequest;
use proto::ProtoSqliteRequest_oneof_request;
use proto::ProtoSqliteResponse;
use proto::ProtoSqliteResponse_oneof_response;
use proto::ProtoSqliteQuery;
use proto::ProtoSqliteQuery_oneof_query;
use proto::ProtoSqliteQueryResponse;
use proto::ProtoSqliteQueryResponse_oneof_response;
use proto::ProtoSqliteExecute;
use proto::ProtoSqliteExecute_oneof_execute;
use proto::ProtoSqliteExecuteResponse;
use proto::ProtoSqliteExecuteResponse_oneof_response;
use request::SqliteRequest;
use request::SqliteResponse;
use request::SqliteQuery;
use request::SqliteQueryResponse;
use request::SqliteExecute;
use request::SqliteExecuteResponse;

impl From<SqliteRequest> for ProtoSqliteRequest {
    fn from(sqlite_request: SqliteRequest) -> Self {
        let mut proto_sqlite_request = ProtoSqliteRequest::new();
        match sqlite_request {
            SqliteRequest::Query(sqlite_query) =>
                proto_sqlite_request.set_query(sqlite_query.into()),
            SqliteRequest::Execute(sqlite_execute) =>
                proto_sqlite_request.set_execute(sqlite_execute.into()),
        }
        proto_sqlite_request
    }
}

impl From<ProtoSqliteRequest> for SqliteRequest {
    fn from(proto_sqlite_request: ProtoSqliteRequest) -> Self {
        match proto_sqlite_request.request.unwrap() {
            ProtoSqliteRequest_oneof_request::query(proto_sqlite_query) =>
                SqliteRequest::Query(proto_sqlite_query.into()),
            ProtoSqliteRequest_oneof_request::execute(proto_sqlite_execute) =>
                SqliteRequest::Execute(proto_sqlite_execute.into()),
        }
    }
}

impl From<SqliteResponse> for ProtoSqliteResponse {
    fn from(sqlite_response: SqliteResponse) -> Self {
        let mut proto_sqlite_response = ProtoSqliteResponse::new();
        match sqlite_response {
            SqliteResponse::Query(sqlite_query_response) =>
                proto_sqlite_response.set_query(sqlite_query_response.into()),
            SqliteResponse::Execute(sqlite_execute_response) =>
                proto_sqlite_response.set_execute(sqlite_execute_response.into()),
        }
        proto_sqlite_response
    }
}

impl From<ProtoSqliteResponse> for SqliteResponse {
    fn from(proto_sqlite_response: ProtoSqliteResponse) -> Self {
        match proto_sqlite_response.response.unwrap() {
            ProtoSqliteResponse_oneof_response::query(proto_sqlite_query_response) =>
                SqliteResponse::Query(proto_sqlite_query_response.into()),
            ProtoSqliteResponse_oneof_response::execute(proto_sqlite_execute_response) =>
                SqliteResponse::Execute(proto_sqlite_execute_response.into()),
        }
    }
}

impl From<SqliteQuery> for ProtoSqliteQuery {
    fn from(sqlite_query: SqliteQuery) -> Self {
        let mut proto_sqlite_query = ProtoSqliteQuery::new();
        match sqlite_query {
            SqliteQuery::Single(query) =>
                proto_sqlite_query.set_single(query.into()),
            SqliteQuery::Bulk(bulk_query) =>
                proto_sqlite_query.set_bulk(bulk_query.into()),
        }
        proto_sqlite_query
    }
}

impl From<ProtoSqliteQuery> for SqliteQuery {
    fn from(proto_sqlite_query: ProtoSqliteQuery) -> Self {
        match proto_sqlite_query.query.unwrap() {
            ProtoSqliteQuery_oneof_query::single(proto_query) =>
                SqliteQuery::Single(proto_query.into()),
            ProtoSqliteQuery_oneof_query::bulk(proto_bulk_query) =>
                SqliteQuery::Bulk(proto_bulk_query.into()),
        }
    }
}

impl From<SqliteQueryResponse> for ProtoSqliteQueryResponse {
    fn from(sqlite_query_response: SqliteQueryResponse) -> Self {
        let mut proto_sqlite_query = ProtoSqliteQueryResponse::new();
        match sqlite_query_response {
            SqliteQueryResponse::Single(query_response) =>
                proto_sqlite_query.set_single(query_response.into()),
            SqliteQueryResponse::Bulk(bulk_query_response) =>
                proto_sqlite_query.set_bulk(bulk_query_response.into()),
        }
        proto_sqlite_query
    }
}

impl From<ProtoSqliteQueryResponse> for SqliteQueryResponse {
    fn from(proto_sqlite_query_response: ProtoSqliteQueryResponse) -> Self {
        match proto_sqlite_query_response.response.unwrap() {
            ProtoSqliteQueryResponse_oneof_response::single(proto_query_response) =>
                SqliteQueryResponse::Single(proto_query_response.into()),
            ProtoSqliteQueryResponse_oneof_response::bulk(proto_bulk_query_response) =>
                SqliteQueryResponse::Bulk(proto_bulk_query_response.into()),
        }
    }
}

impl From<SqliteExecute> for ProtoSqliteExecute {
    fn from(sqlite_execute: SqliteExecute) -> Self {
        let mut proto_sqlite_execute = ProtoSqliteExecute::new();
        match sqlite_execute {
            SqliteExecute::Single(execute) =>
                proto_sqlite_execute.set_single(execute.into()),
            SqliteExecute::Bulk(bulk_execute) =>
                proto_sqlite_execute.set_bulk(bulk_execute.into()),
        }
        proto_sqlite_execute
    }
}

impl From<ProtoSqliteExecute> for SqliteExecute {
    fn from(proto_sqlite_execute: ProtoSqliteExecute) -> Self {
        match proto_sqlite_execute.execute.unwrap() {
            ProtoSqliteExecute_oneof_execute::single(proto_execute) =>
                SqliteExecute::Single(proto_execute.into()),
            ProtoSqliteExecute_oneof_execute::bulk(proto_bulk_execute) =>
                SqliteExecute::Bulk(proto_bulk_execute.into()),
        }
    }
}

impl From<SqliteExecuteResponse> for ProtoSqliteExecuteResponse {
    fn from(sqlite_execute_response: SqliteExecuteResponse) -> Self {
        let mut proto_sqlite_execute = ProtoSqliteExecuteResponse::new();
        match sqlite_execute_response {
            SqliteExecuteResponse::Single(execute_response) =>
                proto_sqlite_execute.set_single(execute_response.into()),
            SqliteExecuteResponse::Bulk(bulk_execute_response) =>
                proto_sqlite_execute.set_bulk(bulk_execute_response.into()),
        }
        proto_sqlite_execute
    }
}

impl From<ProtoSqliteExecuteResponse> for SqliteExecuteResponse {
    fn from(proto_sqlite_execute_response: ProtoSqliteExecuteResponse) -> Self {
        match proto_sqlite_execute_response.response.unwrap() {
            ProtoSqliteExecuteResponse_oneof_response::single(proto_execute_response) =>
                SqliteExecuteResponse::Single(proto_execute_response.into()),
            ProtoSqliteExecuteResponse_oneof_response::bulk(proto_bulk_execute_response) =>
                SqliteExecuteResponse::Bulk(proto_bulk_execute_response.into()),
        }
    }
}
