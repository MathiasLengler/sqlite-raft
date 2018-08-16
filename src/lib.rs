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

/// TODO:
///
/// Sanitize input using sqlpop:
/// match SELECT / INSERT
/// inline random()/etc.
///
/// serde/protobuf

/// Bulk execution of a series of SQL commands. Each command can have a queue of parameters.
#[derive(Debug, Clone, PartialEq)]
pub enum BulkSqliteCommand {
    BulkExecute(BulkExecute),
    BulkQuery(BulkQuery),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SingleSqliteCommand {
    /// Execute a statement once or multiple times with different parameters.
    Execute(Execute),
    /// Execute a query once or multiple times with different parameters.
    Query(Query),
}

