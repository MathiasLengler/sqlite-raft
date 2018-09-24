use error::Result;
use model::core::CoreId;
use raft::eraftpb::Snapshot;
use raft::eraftpb::SnapshotMetadata;
use rusqlite::Transaction;
use self::node::SqliteConfState;
use self::raw_snapshot::RawSqliteSnapshot;

pub mod node;
mod raw_snapshot;

pub struct SqliteSnapshot {
    raw_snapshot: RawSqliteSnapshot,
    conf_state: SqliteConfState,
}

impl SqliteSnapshot {
    pub fn insert_or_replace(&self, tx: &Transaction, core_id: CoreId) -> Result<()> {
        self.raw_snapshot.insert_or_replace(&tx, core_id)?;
        self.conf_state.insert_or_replace(&tx, core_id)?;
        Ok(())
    }
    pub fn query(tx: &Transaction, core_id: CoreId) -> Result<SqliteSnapshot> {
        Ok(SqliteSnapshot {
            raw_snapshot: RawSqliteSnapshot::query(&tx, core_id)?,
            conf_state: SqliteConfState::query(&tx, core_id)?,
        })
    }
}

impl From<Snapshot> for SqliteSnapshot {
    fn from(mut snapshot: Snapshot) -> Self {
        let mut metadata = snapshot.take_metadata();

        SqliteSnapshot {
            raw_snapshot: RawSqliteSnapshot {
                data: snapshot.take_data(),
                index: metadata.get_index() as i64,
                term: metadata.get_term() as i64,
            },
            conf_state: metadata.take_conf_state().into(),
        }
    }
}

impl From<SqliteSnapshot> for Snapshot {
    fn from(sqlite_snapshot: SqliteSnapshot) -> Self {
        let SqliteSnapshot { raw_snapshot, conf_state, } = sqlite_snapshot;

        let mut metadata = SnapshotMetadata::new();
        metadata.set_conf_state(conf_state.into());
        metadata.set_index(raw_snapshot.index as u64);
        metadata.set_term(raw_snapshot.term as u64);

        let mut snapshot = Snapshot::new();
        snapshot.set_data(raw_snapshot.data);
        snapshot.set_metadata(metadata);
        snapshot
    }
}

impl Default for SqliteSnapshot {
    fn default() -> Self {
        let snapshot = Snapshot::new();

        snapshot.into()
    }
}
