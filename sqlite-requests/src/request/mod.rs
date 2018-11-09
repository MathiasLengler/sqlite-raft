use connection::access::Access;
use connection::access::ReadAccess;
use connection::access::WriteAccess;
use connection::AccessConnectionRef;
use error::Result;
use execute::BulkExecute;
use execute::Execute;
use execute::ExecuteResult;
use query::BulkQuery;
use query::Query;
use query::QueryResultSet;

mod proto_convert;


/// Implemented by every SQLite request.
///
/// Used by `AccessConnection::run`.
pub trait Request<A: Access> {
    /// The returned response type when running this query.
    type Response;

    fn apply_to_conn(&self, conn: &AccessConnectionRef<A>) -> Result<Self::Response>;
}


/// Every possible SQLite request.
/// Used as a serialization root point for transferring or persisting SQLite requests.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SqliteRequest {
    Query(SqliteQuery),
    Execute(SqliteExecute),
}

impl<A: ReadAccess + WriteAccess> Request<A> for SqliteRequest {
    type Response = SqliteResponse;

    fn apply_to_conn(&self, conn: &AccessConnectionRef<A>) -> Result<Self::Response> {
        Ok(match self {
            SqliteRequest::Query(sqlite_query) =>
                SqliteResponse::Query(sqlite_query.apply_to_conn(conn)?),
            SqliteRequest::Execute(sqlite_execute) =>
                SqliteResponse::Execute(sqlite_execute.apply_to_conn(conn)?),
        })
    }
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
/// A query is a SQL-Request which can't modify the DB and requires `ReadAccess` to be run, e.g. a `SELECT` statement.
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

impl<A: ReadAccess> Request<A> for SqliteQuery {
    type Response = SqliteQueryResponse;

    fn apply_to_conn(&self, conn: &AccessConnectionRef<A>) -> Result<Self::Response> {
        Ok(match self {
            SqliteQuery::Single(query) =>
                SqliteQueryResponse::Single(query.apply_to_conn(conn)?),
            SqliteQuery::Bulk(bulk_query) =>
                SqliteQueryResponse::Bulk(bulk_query.apply_to_conn(conn)?),
        })
    }
}

/// A single SQLite statement or a series of them.
/// A statement is a SQL-Request which can modify the DB and requires `WriteAccess` to be run, e.g. everything but a query.
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

impl<A: WriteAccess> Request<A> for SqliteExecute {
    type Response = SqliteExecuteResponse;

    fn apply_to_conn(&self, conn: &AccessConnectionRef<A>) -> Result<Self::Response> {
        Ok(match self {
            SqliteExecute::Single(execute) =>
                SqliteExecuteResponse::Single(execute.apply_to_conn(conn)?),
            SqliteExecute::Bulk(bulk_execute) =>
                SqliteExecuteResponse::Bulk(bulk_execute.apply_to_conn(conn)?),
        })
    }
}
