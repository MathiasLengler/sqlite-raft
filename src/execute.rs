use connection::AccessConnection;
use connection::AccessTransaction;
use connection::ReadWrite;
use error::Result;
use parameter::IndexedParameters;
use parameter::NamedParameters;
use parameter::QueuedParameters;
use rusqlite::Statement;
use rusqlite::types::ToSql;

#[derive(Debug, Clone, PartialEq)]
pub struct BulkExecute {
    executes: Vec<Execute>,
}

impl BulkExecute {
    pub fn new(executes: Vec<Execute>) -> BulkExecute {
        BulkExecute {
            executes,
        }
    }

    pub fn apply_to_conn(&self, conn: &mut AccessConnection<ReadWrite>) -> Result<Vec<Vec<ExecuteResult>>> {
        conn.inside_transaction(|tx| {
            self.executes.iter().map(|execute| {
                execute.apply_to_tx(tx)
            }).collect::<Result<Vec<_>>>()
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Execute {
    sql: String,
    queued_parameters: QueuedParameters,
}

impl Execute {
    pub fn new_indexed(sql: &str, queued_indexed_parameters: &[&[&ToSql]]) -> Result<Execute> {
        Ok(Execute {
            sql: sql.to_string(),
            queued_parameters: QueuedParameters::new_indexed(queued_indexed_parameters)?,
        })
    }

    pub fn new_named(sql: &str, queued_named_parameters: &[&[(&str, &ToSql)]]) -> Result<Execute> {
        Ok(Execute {
            sql: sql.to_string(),
            queued_parameters: QueuedParameters::new_named(queued_named_parameters)?,
        })
    }

    pub fn apply_to_conn(&self, conn: &mut AccessConnection<ReadWrite>) -> Result<Vec<ExecuteResult>> {
        conn.inside_transaction(|tx| {
            self.apply_to_tx(tx)
        })
    }

    fn apply_to_tx(&self, tx: &mut AccessTransaction<ReadWrite>) -> Result<Vec<ExecuteResult>> {
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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ExecuteResult {
    changes: i32
}

impl ExecuteResult {
    /// The number of rows that were changed or inserted or deleted.
    pub fn changes(&self) -> i32 {
        self.changes
    }
}