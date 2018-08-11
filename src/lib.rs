#[macro_use]
extern crate failure;
extern crate rusqlite;


use connection::AccessConnection;
use connection::ReadOnly;
use connection::ReadWrite;
use error::Result;
use rusqlite::Row;
use rusqlite::types::ToSql;
use rusqlite::types::Value;
use std::result;
use connection::AccessTransaction;
use rusqlite::Statement;
use parameter::QueuedParameters;
use parameter::IndexedParameters;
use parameter::NamedParameters;

pub mod connection;
pub mod error;
pub mod parameter;

/// TODO:
///
/// Sanitize input using sqlpop:
/// match SELECT / INSERT
/// inline random()/etc.
///
/// serde/protobuf

/// Bulk execution of a series of SQL commands. Each command can have a queue of parameters.
#[derive(Debug, Clone, PartialEq)]
pub enum BulkSqliteCommands {
    BulkExecute(BulkExecute),
    BulkQuery(BulkQuery),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BulkExecute {
    executes: Vec<Execute>,
}

impl BulkExecute {
    pub fn apply(&self, conn: &mut AccessConnection<ReadWrite>) -> Result<Vec<Vec<ExecuteResult>>> {
        conn.inside_transaction(|mut tx| {
            self.executes.iter().map(|execute| {
                execute.apply(&mut tx)
            }).collect::<Result<Vec<_>>>()
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BulkQuery {
    queries: Vec<Query>,
}

impl BulkQuery {
    fn apply(&self, conn: &mut AccessConnection<ReadOnly>) -> Result<Vec<Vec<QueryResult>>> {
        conn.inside_transaction(|mut tx| {
            self.queries.iter().map(|query| {
                query.apply(&mut tx)
            }).collect::<Result<Vec<_>>>()
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SqliteCommand {
    /// Execute a statement once or multiple times with different parameters.
    Execute(Execute),
    /// Execute a query once or multiple times with different parameters.
    Query(Query),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Execute {
    sql: String,
    queued_parameters: QueuedParameters,
}

impl Execute {
    fn apply(&self, tx: &mut AccessTransaction<ReadWrite>) -> Result<Vec<ExecuteResult>> {
        let tx = tx.as_mut();
        let mut stmt = tx.prepare(&self.sql)?;

        let res = self.queued_parameters.map_parameter_variants(
            &mut stmt,
            |stmt: &mut Statement, parameters: &IndexedParameters| {
                let changes = stmt.execute(
                    &parameters.as_arg(),
                )?;

                Ok(ExecuteResult {
                    changes,
                })
            },
            |stmt: &mut Statement, parameters: &NamedParameters| {
                let changes = stmt.execute_named(
                    &parameters.as_arg(),
                )?;

                Ok(ExecuteResult {
                    changes,
                })
            },
        );

        res
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExecuteResult {
    /// The number of rows that were changed or inserted or deleted.
    changes: i32
}

#[derive(Debug, Clone, PartialEq)]
pub struct Query {
    sql: String,
    queued_parameters: QueuedParameters,
}

impl Query {
    pub fn new_indexed(sql: &str, queued_indexed_parameters: &[&[&ToSql]]) -> Result<Query> {
        Ok(Query {
            sql: sql.to_string(),
            queued_parameters: QueuedParameters::new_indexed(queued_indexed_parameters)?,
        })
    }

    pub fn new_named(sql: &str, queued_named_parameters: &[&[(&str, &ToSql)]]) -> Result<Query> {
        Ok(Query {
            sql: sql.to_string(),
            queued_parameters: QueuedParameters::new_named(queued_named_parameters)?,
        })
    }

    fn apply(&self, tx: &mut AccessTransaction<ReadOnly>) -> Result<Vec<QueryResult>> {
        let tx = tx.as_mut();
        let mut stmt = tx.prepare(&self.sql)?;

        let res = self.queued_parameters.map_parameter_variants(
            &mut stmt,
            |stmt: &mut Statement, parameters: &IndexedParameters| {
                let rows = stmt.query_map(
                    &parameters.as_arg(),
                    QueryResultRow::query_map_arg(),
                )?;

                QueryResult::try_from(rows)
            },
            |stmt: &mut Statement, parameters: &NamedParameters| {
                let rows = stmt.query_map_named(
                    &parameters.as_arg(),
                    QueryResultRow::query_map_arg(),
                )?;

                QueryResult::try_from(rows)
            },
        );

        res
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryResult {
    rows: Vec<QueryResultRow>,
}

impl QueryResult {
    fn try_from(rows_iter: impl Iterator<Item=result::Result<QueryResultRow, rusqlite::Error>>)
                -> Result<QueryResult> {
        let rows: result::Result<Vec<QueryResultRow>, rusqlite::Error> = rows_iter.collect();

        Ok(QueryResult {
            rows: rows?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryResultRow {
    row: Vec<Value>
}

impl QueryResultRow {
    fn query_map_arg() -> impl FnMut(&Row) -> QueryResultRow {
        |row: &Row| {
            let row: Vec<_> = (0..row.column_count())
                .map(|row_index| row.get(row_index)).collect();
            QueryResultRow {
                row,
            }
        }
    }
}
