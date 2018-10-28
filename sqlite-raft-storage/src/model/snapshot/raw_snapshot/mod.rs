use error::Result;
use model::core::CoreId;
use model::core::CoreTx;
use model::snapshot::raw_snapshot::metadata::SqliteSnapshotMetadata;
use rusqlite::Row;
use rusqlite::types::ToSql;

pub mod metadata;

pub struct RawSqliteSnapshot {
    pub data: Vec<u8>,
    pub metadata: SqliteSnapshotMetadata,
}

impl RawSqliteSnapshot {
    const SQL_QUERY: &'static str =
        include_str!("../../../../res/sql/snapshot/query.sql");
    const SQL_INSERT_OR_REPLACE: &'static str =
        include_str!("../../../../res/sql/snapshot/insert_or_replace.sql");

    fn as_named_params<'a>(&'a self, core_id: &'a CoreId) -> [(&'static str, &'a dyn ToSql); 4] {
        [
            (":data", &self.data),
            (":index", &self.metadata.index),
            (":term", &self.metadata.term),
            core_id.as_named_param(),
        ]
    }

    fn from_row(row: &Row) -> Self {
        // TODO: get_checked

        RawSqliteSnapshot {
            data: row.get("data"),
            metadata: SqliteSnapshotMetadata::from_row(row),
        }
    }

    pub fn query(core_tx: &CoreTx) -> Result<RawSqliteSnapshot> {
        core_tx.tx().query_row_named(
            RawSqliteSnapshot::SQL_QUERY,
            &[core_tx.core_id().as_named_param()],
            RawSqliteSnapshot::from_row,
        ).map_err(Into::into)
    }

    pub(super) fn insert_or_replace(&self, core_tx: &CoreTx) -> Result<()> {
        core_tx.tx().execute_named(RawSqliteSnapshot::SQL_INSERT_OR_REPLACE, &self.as_named_params(&core_tx.core_id()))?;

        Ok(())
    }
}
