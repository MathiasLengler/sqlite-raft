//! # TODO
//!
//! ## Sanitize input using sqlpop:
//! match SELECT / INSERT
//! inline random()/etc.
//!
//! ## Serialization
//! - protobuf

#[macro_use]
extern crate failure;
extern crate rusqlite;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate protobuf;

use execute::BulkExecute;
use execute::Execute;
use query::BulkQuery;
use query::Query;

pub mod connection;
pub mod error;
pub mod parameter;
pub mod query;
pub mod execute;
pub mod proto;
mod value;

// TODO: Naming: SqliteQuery/Query vs Query/SingleQuery

/// Every possible SQLite command. Used as a serialization root point for transferring SQLite commands.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SqliteCommand {
    Execute(SqliteExecute),
    Query(SqliteQuery),
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

/// A single SQLite query or a series of them.
/// A query is a SQL-Command which can't modify the DB and requires `ReadOnly` access to be run, e.g. a `SELECT` statement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SqliteQuery {
    /// Execute a single query once or multiple times with different parameters.
    Single(Query),
    /// Execute a series of queries. Each query can be run with multiple different parameters.
    Bulk(BulkQuery),
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