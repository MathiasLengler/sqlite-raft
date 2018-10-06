use error::Result;
use model::core::CoreId;
use rusqlite::Row;
use rusqlite::Transaction;

pub struct SqliteSnapshotMetadata {
    pub index: i64,
    pub term: i64,
}

impl SqliteSnapshotMetadata {
    const SQL_QUERY: &'static str =
        include_str!("../../../../res/sql/snapshot/query_metadata.sql");

    pub(super) fn from_row(row: &Row) -> Self {
        SqliteSnapshotMetadata {
            index: row.get("index"),
            term: row.get("term"),
        }
    }

    pub fn query(tx: &Transaction, core_id: CoreId) -> Result<SqliteSnapshotMetadata> {
        tx.query_row_named(
            SqliteSnapshotMetadata::SQL_QUERY,
            &[core_id.as_named_param()],
            SqliteSnapshotMetadata::from_row,
        ).map_err(Into::into)
    }
}
