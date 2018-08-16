//! # TODO
//!
//! ## Sanitize input using sqlpop:
//! match SELECT / INSERT
//! inline random()/etc.
//!
//! ## Serialization
//! - serde
//! - protobuf

#[macro_use]
extern crate failure;
extern crate rusqlite;

use execute::BulkExecute;
use execute::Execute;
use query::BulkQuery;
use query::Query;

pub mod connection;
pub mod error;
pub mod parameter;
pub mod query;
pub mod execute;

/// Every possible SQLite command. Used as a serialization root point for transferring SQLite commands.
#[derive(Debug, Clone, PartialEq)]
pub enum SqliteCommand {
    Execute(SqliteExecute),
    Query(SqliteQuery),
}

/// A single SQLite query or a series of them.
/// A query is a SQL-Command which can't modify the DB and requires `ReadOnly` access to be run, e.g. a `SELECT` statement.
#[derive(Debug, Clone, PartialEq)]
pub enum SqliteQuery {
    /// Execute a single query once or multiple times with different parameters.
    Single(Query),
    /// Execute a series of queries. Each query can be run with multiple different parameters.
    Bulk(BulkQuery),
}

/// A single SQLite statement or a series of them.
/// A statement is a SQL-Command which can modify the DB and requires `ReadWrite` access to be run, e.g. everything but a query.
#[derive(Debug, Clone, PartialEq)]
pub enum SqliteExecute {
    /// Execute a single statement once or multiple times with different parameters.
    Single(Execute),
    /// Execute a series of statements. Each statement can be run with multiple different parameters.
    Bulk(BulkExecute),
}