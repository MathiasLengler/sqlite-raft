//! # TODO
//!
//! ## Sanitize input using sqlpop:
//! match SELECT / INSERT
//! inline random()/etc.

#[macro_use]
extern crate failure;
extern crate protobuf;
extern crate rusqlite;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use connection::AccessTransaction;
use connection::Command;
use connection::ReadOnly;
use connection::ReadWrite;
use error::Result;
use execute::BulkExecute;
use execute::Execute;
use execute::ExecuteResult;
use query::BulkQuery;
use query::Query;
use query::QueryResult;

pub mod connection;
pub mod error;
pub mod parameter;
pub mod query;
pub mod execute;
pub mod proto;
mod value;

// TODO: move to modules (command?)

// TODO: Naming: SqliteQuery/Query vs Query/SingleQuery
// TODO: Naming: (Sqlite)Request vs (Sqlite)Command
// TODO: Naming: Response vs Result
// TODO: Naming: "Command" trait

/// Every possible SQLite command.
/// Used as a serialization root point for transferring or persisting SQLite commands.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SqliteCommand {
    Query(SqliteQuery),
    Execute(SqliteExecute),
}

impl From<Query> for SqliteCommand {
    fn from(query: Query) -> Self {
        SqliteCommand::Query(SqliteQuery::Single(query))
    }
}

impl From<BulkQuery> for SqliteCommand {
    fn from(bulk_query: BulkQuery) -> Self {
        SqliteCommand::Query(SqliteQuery::Bulk(bulk_query))
    }
}

impl From<Execute> for SqliteCommand {
    fn from(execute: Execute) -> Self {
        SqliteCommand::Execute(SqliteExecute::Single(execute))
    }
}

impl From<BulkExecute> for SqliteCommand {
    fn from(bulk_execute: BulkExecute) -> Self {
        SqliteCommand::Execute(SqliteExecute::Bulk(bulk_execute))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SqliteCommandResult {
    Query(SqliteQueryResult),
    Execute(SqliteExecuteResult),
}

/// A single SQLite query or a series of them.
/// A query is a SQL-Command which can't modify the DB and requires `ReadOnly` access to be run, e.g. a `SELECT` statement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SqliteQuery {
    /// Execute a single query once or multiple times with different parameters.
    Single(Query),
    /// Execute a series of queries. Each query can be run with multiple different parameters.
    Bulk(BulkQuery),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SqliteQueryResult {
    Single(Vec<QueryResult>),
    Bulk(Vec<Vec<QueryResult>>),
}

impl Command for SqliteQuery {
    type Access = ReadOnly;
    type Return = SqliteQueryResult;

    fn apply_to_tx(&self, tx: &mut AccessTransaction<Self::Access>) -> Result<Self::Return> {
        Ok(match self {
            SqliteQuery::Single(query) =>
                SqliteQueryResult::Single(query.apply_to_tx(tx)?),
            SqliteQuery::Bulk(bulk_query) =>
                SqliteQueryResult::Bulk(bulk_query.apply_to_tx(tx)?),
        })
    }
}

/// A single SQLite statement or a series of them.
/// A statement is a SQL-Command which can modify the DB and requires `ReadWrite` access to be run, e.g. everything but a query.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SqliteExecute {
    /// Execute a single statement once or multiple times with different parameters.
    Single(Execute),
    /// Execute a series of statements. Each statement can be run with multiple different parameters.
    Bulk(BulkExecute),
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SqliteExecuteResult {
    Single(Vec<ExecuteResult>),
    Bulk(Vec<Vec<ExecuteResult>>),
}

impl Command for SqliteExecute {
    type Access = ReadWrite;
    type Return = SqliteExecuteResult;

    fn apply_to_tx(&self, tx: &mut AccessTransaction<Self::Access>) -> Result<Self::Return> {
        Ok(match self {
            SqliteExecute::Single(execute) =>
                SqliteExecuteResult::Single(execute.apply_to_tx(tx)?),
            SqliteExecute::Bulk(bulk_execute) =>
                SqliteExecuteResult::Bulk(bulk_execute.apply_to_tx(tx)?),
        })
    }
}
