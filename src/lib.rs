extern crate rusqlite;
#[macro_use]
extern crate failure;


pub mod connection;
pub mod error;

use error::Result;

use rusqlite::types::Value;
use connection::AccessConnection;
use connection::ReadOnly;
use rusqlite::types::ToSql;

/// TODO:
///
/// Sanitize input using sqlpop:
/// match SELECT / INSERT
/// inline random()/etc.
///
/// serde


pub enum SqliteCommand {
    /// Execute multiple distinct statements.
    BatchExecute(Vec<Execute>),
    /// Execute a statement once or multiple times with different parameters.
    Execute(Execute),
    /// Execute a query once or multiple times with different parameters.
    Query(Query),
}

pub struct Execute {
    sql: String,
    queued_parameters: QueuedParameters,
}

pub struct Query {
    sql: String,
    queued_parameters: QueuedParameters,
}

impl Query {
    // TODO: refactor duplicate logic between Query/Execute
    fn apply(&self, conn: &mut AccessConnection<ReadOnly>) -> Result<QueryResult> {
        let conn = conn.inner_mut();

        let tx = conn.transaction()?;
        let mut stmt = tx.prepare(&self.sql)?;

        match self.queued_parameters {
            QueuedParameters::Indexed(ref indexed_parameters) => {
                for parameters in indexed_parameters {
                    let rows = stmt.query_map(
                        &parameters.as_arg(),
                        // TODO: inject QueryResult mapping function
                        |row| {},
                    )?;

                    // TODO: build QueryResult
                }
            }
            QueuedParameters::Named(_) => {}
        };

        Ok(QueryResult {})
    }
}

pub struct QueryResult {}

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
