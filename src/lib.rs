//! # TODO
//!
//! ## Sanitize input using sqlpop:
//! match SELECT / INSERT
//! inline random()/etc.
//!
//! ## Serializaction
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

#[derive(Debug, Clone, PartialEq)]
pub enum SqliteCommand {
    Single(SingleSqliteCommand),
    Bulk(BulkSqliteCommand),
}


#[derive(Debug, Clone, PartialEq)]
pub enum SingleSqliteCommand {
    /// Execute a statement once or multiple times with different parameters.
    Execute(Execute),
    /// Execute a query once or multiple times with different parameters.
    Query(Query),
}

/// Bulk execution of a series of SQL commands. Each command can have a queue of parameters.
#[derive(Debug, Clone, PartialEq)]
pub enum BulkSqliteCommand {
    BulkExecute(BulkExecute),
    BulkQuery(BulkQuery),
}

// TODO: decide between the tow structures
enum TestSqliteCommand {
    Execute(SqliteExecute),
    Query(SqliteQuery),
}

enum SqliteQuery {
    Single(Query),
    Bulk(BulkQuery),
}

enum SqliteExecute {
    Single(Execute),
    Bulk(BulkExecute),
}