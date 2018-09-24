use error::Result;
use model::core::CoreId;
use rusqlite::Row;
use rusqlite::Transaction;
use rusqlite::types::ToSql;

pub struct RawSqliteSnapshot {
    pub data: Vec<u8>,
    pub index: i64,
    pub term: i64,
}

impl RawSqliteSnapshot {
    const SQL_QUERY: &'static str =
        include_str!("../../../res/sql/snapshot/query.sql");
    const SQL_INSERT_OR_REPLACE: &'static str =
        include_str!("../../../res/sql/snapshot/insert_or_replace.sql");

    pub fn as_named_params<'a>(&'a self, core_id: &'a CoreId) -> [(&'static str, &'a ToSql); 4] {
        [
            (":data", &self.data),
            (":index", &self.index),
            (":term", &self.term),
            core_id.as_named_param(),
        ]
    }

    fn from_row(row: &Row) -> Self {
        // TODO: get_checked

        RawSqliteSnapshot {
            data: row.get("data"),
            index: row.get("index"),
            term: row.get("term"),
        }
    }

    pub fn query(tx: &Transaction, core_id: CoreId) -> Result<RawSqliteSnapshot> {
        tx.query_row_named(
            RawSqliteSnapshot::SQL_QUERY,
            &[core_id.as_named_param()],
            RawSqliteSnapshot::from_row,
        ).map_err(Into::into)
    }

    pub fn insert_or_replace(&self, tx: &Transaction, core_id: CoreId) -> Result<()> {
        tx.execute_named(RawSqliteSnapshot::SQL_INSERT_OR_REPLACE, &self.as_named_params(&core_id))?;

        Ok(())
    }
}
