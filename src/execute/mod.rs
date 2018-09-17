use connection::AccessTransaction;
use request::Request;
use connection::ReadWrite;
use error::Result;
use parameter::IndexedParameters;
use parameter::NamedParameters;
use parameter::QueuedParameters;
use rusqlite::Statement;
use rusqlite::types::ToSql;

mod proto_convert;

/// Execute a series of statements. Each statement can be run with multiple different parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BulkExecute {
    executes: Vec<Execute>,
}

impl BulkExecute {
    pub fn new(executes: Vec<Execute>) -> BulkExecute {
        BulkExecute {
            executes,
        }
    }
}

impl Request for BulkExecute {
    type Access = ReadWrite;
    type Response = Vec<Vec<ExecuteResult>>;

    fn apply_to_tx(&self, tx: &mut AccessTransaction<Self::Access>) -> Result<Self::Response> {
        self.executes.iter().map(|execute| {
            execute.apply_to_tx(tx)
        }).collect::<Result<Vec<_>>>()
    }
}


/// Execute a single statement once or multiple times with different parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
}

impl Request for Execute {
    type Access = ReadWrite;
    type Response = Vec<ExecuteResult>;

    fn apply_to_tx(&self, tx: &mut AccessTransaction<Self::Access>) -> Result<Self::Response> {
        let tx = tx.as_mut_inner();
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


#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExecuteResult {
    changes: usize
}

impl ExecuteResult {
    /// The number of rows that were changed or inserted or deleted.
    pub fn changes(&self) -> usize {
        self.changes
    }
}