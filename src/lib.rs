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

pub mod connection;
pub mod error;

/// TODO:
///
/// Sanitize input using sqlpop:
/// match SELECT / INSERT
/// inline random()/etc.
///
/// serde

/// Bulk execution of a series of SQL commands. Each command can have a queue of parameters.
pub enum BulkSqliteCommands {
    BulkExecute(BulkExecute),
    BulkQuery(BulkQuery),
}

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

pub struct ExecuteResult {
    /// The number of rows that were changed or inserted or deleted.
    changes: i32
}

pub struct Query {
    sql: String,
    queued_parameters: QueuedParameters,
}

impl Query {
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

impl QueuedParameters {
    fn new() -> QueuedParameters {
        // TODO: ensure at least one parameter set in each variant.
        unimplemented!()
    }

    fn map_parameter_variants<T>(&self,
                                 stmt: &mut Statement,
                                 mut indexed: impl FnMut(&mut Statement, &IndexedParameters) -> Result<T>,
                                 mut named: impl FnMut(&mut Statement, &NamedParameters) -> Result<T>)
                                 -> Result<Vec<T>> {
        match self {
            QueuedParameters::Indexed(ref queued_indexed_parameters) => {
                queued_indexed_parameters.iter().map(|parameters| {
                    indexed(stmt, parameters)
                }).collect()
            }
            QueuedParameters::Named(ref queued_named_parameters) => {
                queued_named_parameters.iter().map(|parameters| {
                    named(stmt, parameters)
                }).collect()
            }
        }
    }
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
