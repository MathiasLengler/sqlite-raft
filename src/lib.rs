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
    Execute(SqliteExecute),
    Query(SqliteQuery),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SqliteQuery {
    /// Execute a single query once or multiple times with different parameters.
    Single(Query),
    Bulk(BulkQuery),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SqliteExecute {
    /// Execute a single statement once or multiple times with different parameters.
    Single(Execute),
    Bulk(BulkExecute),
}