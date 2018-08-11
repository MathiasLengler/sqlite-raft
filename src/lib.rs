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

pub mod connection;
pub mod error;

/// TODO:
///
/// Sanitize input using sqlpop:
/// match SELECT / INSERT
/// inline random()/etc.
///
/// serde


pub enum BulkSqliteCommands {
    BulkExecute(BulkExecute),
    BulkQuery(BulkQuery),
}

pub struct BulkExecute {
    executes: Vec<Execute>,
}

impl BulkExecute {
    fn apply(&self, conn: &mut AccessConnection<ReadWrite>) -> Result<Vec<Vec<ExecuteResult>>> {
        let mut tx = conn.access_transaction()?;

        self.executes.iter().map(|execute| {
            execute.apply(&mut tx)
        }).collect()
    }
}

pub struct BulkQuery {
    queries: Vec<Query>,
}

impl BulkQuery {
    fn apply(&self, conn: &mut AccessConnection<ReadOnly>) -> Result<Vec<Vec<QueryResult>>> {
        let mut tx = conn.access_transaction()?;

        self.queries.iter().map(|query| {
            query.apply(&mut tx)
        }).collect()
    }
}

pub enum SqliteCommand {
    /// Execute a statement once or multiple times with different parameters.
    Execute(Execute),
    /// Execute a query once or multiple times with different parameters.
    Query(Query),
}

pub struct Execute {
    sql: String,
    queued_parameters: QueuedParameters,
}

impl Execute {
    // TODO: refactor duplicate logic between Query/Execute
    fn apply(&self, tx: &mut AccessTransaction<ReadWrite>) -> Result<Vec<ExecuteResult>> {
        let tx = tx.inner_mut();
        let mut stmt = tx.prepare(&self.sql)?;

        let res = match self.queued_parameters {
            QueuedParameters::Indexed(ref queued_indexed_parameters) => {
                queued_indexed_parameters.iter().map(|parameters| {
                    let changes = stmt.execute(
                        &parameters.as_arg(),
                    )?;

                    Ok(ExecuteResult {
                        changes,
                    })
                }).collect()
            }
            QueuedParameters::Named(ref queued_named_parameters) => {
                queued_named_parameters.iter().map(|parameters| {
                    let changes = stmt.execute_named(
                        &parameters.as_arg(),
                    )?;

                    Ok(ExecuteResult {
                        changes,
                    })
                }).collect()
            }
        };

        res
    }
}

pub struct ExecuteResult {
    /// The number of rows that were changed or inserted or deleted.
    changes: i32
}

pub struct Query {
    sql: String,
    parameters: QueuedParameters,
}

impl Query {
    // TODO: refactor duplicate logic between Query/Execute
    fn apply(&self, tx: &mut AccessTransaction<ReadOnly>) -> Result<Vec<QueryResult>> {
        let tx = tx.inner_mut();
        let mut stmt = tx.prepare(&self.sql)?;

        let res = match self.parameters {
            QueuedParameters::Indexed(ref queued_indexed_parameters) => {
                queued_indexed_parameters.iter().map(|parameters| {
                    let rows = stmt.query_map(
                        &parameters.as_arg(),
                        QueryResultRow::query_map_arg(),
                    )?;

                    QueryResult::try_from(rows)
                }).collect()
            }
            QueuedParameters::Named(ref queued_named_parameters) => {
                queued_named_parameters.iter().map(|parameters| {
                    let rows = stmt.query_map_named(
                        &parameters.as_arg(),
                        QueryResultRow::query_map_arg(),
                    )?;

                    QueryResult::try_from(rows)
                }).collect()
            }
        };

        res
    }
}

pub struct QueryResult {
    rows: Vec<QueryResultRow>,
}

impl QueryResult {
    fn try_from(rows_iter: impl Iterator<Item=result::Result<QueryResultRow, rusqlite::Error>>) -> Result<QueryResult> {
        let rows: result::Result<Vec<QueryResultRow>, rusqlite::Error> = rows_iter.collect();

        Ok(QueryResult {
            rows: rows?,
        })
    }
}

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

enum QueuedParameters {
    Indexed(Vec<IndexedParameters>),
    Named(Vec<NamedParameters>),
}

pub struct IndexedParameters {
    parameters: Vec<Value>
}

impl IndexedParameters {
    fn as_arg(&self) -> Vec<&ToSql> {
        self.parameters.iter().map(|value| value as &ToSql).collect()
    }
}

pub struct NamedParameters {
    parameters: Vec<NamedParameter>
}

impl NamedParameters {
    fn as_arg(&self) -> Vec<(&str, &ToSql)> {
        self.parameters.iter().map(
            |NamedParameter {
                 name,
                 value,
             }| {
                (name.as_str(), value as &ToSql)
            }).collect()
    }
}

pub struct NamedParameter {
    name: String,
    value: Value,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
