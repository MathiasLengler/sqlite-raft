use error::Result;
use model::core::CoreTx;
use raft::eraftpb::Snapshot;
use raft::eraftpb::SnapshotMetadata;
use self::node::SqliteConfState;
pub use self::raw_snapshot::metadata::SqliteSnapshotMetadata;
use self::raw_snapshot::RawSqliteSnapshot;

pub mod node;
mod raw_snapshot;

pub struct SqliteSnapshot {
    raw_snapshot: RawSqliteSnapshot,
    conf_state: SqliteConfState,
}

impl SqliteSnapshot {
    pub fn insert_or_replace(&self, core_tx: &CoreTx) -> Result<()> {
        self.raw_snapshot.insert_or_replace(&core_tx)?;
        self.conf_state.insert_or_replace(&core_tx)?;
        Ok(())
    }
    pub fn query(core_tx: &CoreTx) -> Result<SqliteSnapshot> {
        Ok(SqliteSnapshot {
            raw_snapshot: RawSqliteSnapshot::query(&core_tx)?,
            conf_state: SqliteConfState::query(&core_tx)?,
        })
    }
}

impl From<Snapshot> for SqliteSnapshot {
    fn from(mut snapshot: Snapshot) -> Self {
        let mut metadata = snapshot.take_metadata();

        SqliteSnapshot {
            raw_snapshot: RawSqliteSnapshot {
                data: snapshot.take_data(),
                metadata: SqliteSnapshotMetadata {
                    index: metadata.get_index() as i64,
                    term: metadata.get_term() as i64,
                },
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
        metadata.set_index(raw_snapshot.metadata.index as u64);
        metadata.set_term(raw_snapshot.metadata.term as u64);

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
