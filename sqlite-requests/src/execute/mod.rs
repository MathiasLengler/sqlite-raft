use connection::access::WriteAccess;
use connection::AccessConnectionRef;
use error::Result;
use parameter::IndexedParameters;
use parameter::NamedParameters;
use parameter::QueuedParameters;
use request::Request;
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

impl<A: WriteAccess> Request<A> for BulkExecute {
    type Response = Vec<Vec<ExecuteResult>>;

    fn apply_to_conn(&self, conn: &AccessConnectionRef<A>) -> Result<Self::Response> {
        self.executes.iter().map(|execute| {
            execute.apply_to_conn(conn)
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

    /// Patches cached changes to ensure deterministic execution independent of previous executes.
    /// Changes get updated only by: INSERT, UPDATE or DELETE
    /// For other execute statements a `0` is returned instead.
    /// # Example:
    /// natively in rusqlite `CREATE TABLE` returns changes of previous execute,
    /// with this patch it returns `0` instead.
    ///
    /// TODO: evaluate sqlite parser for this differentiation
    fn patch_changes(&self) -> impl Fn(usize) -> usize {
        let pass_changes = self.sql.starts_with("INSERT")
            || self.sql.starts_with("UPDATE")
            || self.sql.starts_with("DELETE");

        move |changes| if pass_changes {
            changes
        } else {
            0
        }
    }
}

impl<A: WriteAccess> Request<A> for Execute {
    type Response = Vec<ExecuteResult>;

    fn apply_to_conn(&self, conn: &AccessConnectionRef<A>) -> Result<Self::Response> {
        let mut stmt = conn.prepare(&self.sql)?;

        let patch_changes = self.patch_changes();

        let res = self.queued_parameters.map_parameter_variants(
            &mut stmt,
            |stmt: &mut Statement, parameters: &IndexedParameters| {
                let changes = stmt.execute(
                    &parameters.as_arg(),
                )?;

                let changes = patch_changes(changes);

                Ok(ExecuteResult {
                    changes,
                })
            },
            |stmt: &mut Statement, parameters: &NamedParameters| {
                let changes = stmt.execute_named(
                    &parameters.as_arg(),
                )?;

                let changes = patch_changes(changes);

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
    pub fn changes(self) -> usize {
        self.changes
    }
}
