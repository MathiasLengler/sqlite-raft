use connection::AccessTransaction;
use connection::Request;
use connection::ReadOnly;
use connection::ReadWrite;
use error::Result;
use execute::BulkExecute;
use execute::Execute;
use execute::ExecuteResult;
use query::BulkQuery;
use query::Query;
use query::QueryResultSet;

mod proto_convert;

/// Every possible SQLite request.
/// Used as a serialization root point for transferring or persisting SQLite requests.
///
/// Does not implement the `Request` trait directly, because each variant requires a different `Access`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SqliteRequest {
    Query(SqliteQuery),
    Execute(SqliteExecute),
}

impl From<Query> for SqliteRequest {
    fn from(query: Query) -> Self {
        SqliteRequest::Query(SqliteQuery::Single(query))
    }
}

impl From<BulkQuery> for SqliteRequest {
    fn from(bulk_query: BulkQuery) -> Self {
        SqliteRequest::Query(SqliteQuery::Bulk(bulk_query))
    }
}

impl From<Execute> for SqliteRequest {
    fn from(execute: Execute) -> Self {
        SqliteRequest::Execute(SqliteExecute::Single(execute))
    }
}

impl From<BulkExecute> for SqliteRequest {
    fn from(bulk_execute: BulkExecute) -> Self {
        SqliteRequest::Execute(SqliteExecute::Bulk(bulk_execute))
    }
}

/// Every possible response when executing variants of `SqliteRequest`.
/// Used as a serialization root point for transferring or persisting SQLite responses.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SqliteResponse {
    Query(SqliteQueryResponse),
    Execute(SqliteExecuteResponse),
}

/// A single SQLite query or a series of them.
/// A query is a SQL-Request which can't modify the DB and requires `ReadOnly` access to be run, e.g. a `SELECT` statement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SqliteQuery {
    Single(Query),
    Bulk(BulkQuery),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SqliteQueryResponse {
    Single(Vec<QueryResultSet>),
    Bulk(Vec<Vec<QueryResultSet>>),
}

impl Request for SqliteQuery {
    type Access = ReadOnly;
    type Response = SqliteQueryResponse;

    fn apply_to_tx(&self, tx: &mut AccessTransaction<Self::Access>) -> Result<Self::Response> {
        Ok(match self {
            SqliteQuery::Single(query) =>
                SqliteQueryResponse::Single(query.apply_to_tx(tx)?),
            SqliteQuery::Bulk(bulk_query) =>
                SqliteQueryResponse::Bulk(bulk_query.apply_to_tx(tx)?),
        })
    }
}

/// A single SQLite statement or a series of them.
/// A statement is a SQL-Request which can modify the DB and requires `ReadWrite` access to be run, e.g. everything but a query.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SqliteExecute {
    Single(Execute),
    Bulk(BulkExecute),
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SqliteExecuteResponse {
    Single(Vec<ExecuteResult>),
    Bulk(Vec<Vec<ExecuteResult>>),
}

impl Request for SqliteExecute {
    type Access = ReadWrite;
    type Response = SqliteExecuteResponse;

    fn apply_to_tx(&self, tx: &mut AccessTransaction<Self::Access>) -> Result<Self::Response> {
        Ok(match self {
            SqliteExecute::Single(execute) =>
                SqliteExecuteResponse::Single(execute.apply_to_tx(tx)?),
            SqliteExecute::Bulk(bulk_execute) =>
                SqliteExecuteResponse::Bulk(bulk_execute.apply_to_tx(tx)?),
        })
    }
}
