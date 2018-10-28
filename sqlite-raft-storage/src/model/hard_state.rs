use error::Result;
use model::core::CoreId;
use model::core::CoreTx;
use raft::eraftpb::HardState;
use rusqlite::Row;
use rusqlite::types::ToSql;

pub struct SqliteHardState {
    term: i64,
    vote: i64,
    commit: i64,
}

impl SqliteHardState {
    const SQL_QUERY: &'static str =
        include_str!("../../res/sql/hard_state/query.sql");
    const SQL_INSERT_OR_REPLACE: &'static str =
        include_str!("../../res/sql/hard_state/insert_or_replace.sql");

    fn as_named_params<'a>(&'a self, core_id: &'a CoreId) -> [(&'static str, &'a dyn ToSql); 4] {
        [
            (":term", &self.term),
            (":vote", &self.vote),
            (":commit", &self.commit),
            core_id.as_named_param(),
        ]
    }

    fn from_row(row: &Row) -> SqliteHardState {
        // TODO: get_checked

        SqliteHardState {
            term: row.get("term"),
            vote: row.get("vote"),
            commit: row.get("commit"),
        }
    }

    pub fn query(core_tx: &CoreTx) -> Result<SqliteHardState> {
        core_tx.tx().query_row_named(
            SqliteHardState::SQL_QUERY,
            &[core_tx.core_id().as_named_param()],
            SqliteHardState::from_row,
        ).map_err(Into::into)
    }

    pub fn insert_or_replace(&self, core_tx: &CoreTx) -> Result<()> {
        core_tx.tx().execute_named(SqliteHardState::SQL_INSERT_OR_REPLACE, &self.as_named_params(&core_tx.core_id()))?;

        Ok(())
    }
}

impl From<HardState> for SqliteHardState {
    fn from(hard_state: HardState) -> Self {
        SqliteHardState {
            term: hard_state.get_term() as i64,
            vote: hard_state.get_vote() as i64,
            commit: hard_state.get_commit() as i64,
        }
    }
}

impl From<SqliteHardState> for HardState {
    fn from(sqlite_hard_state: SqliteHardState) -> Self {
        let mut hard_state = HardState::new();
        hard_state.set_term(sqlite_hard_state.term as u64);
        hard_state.set_vote(sqlite_hard_state.vote as u64);
        hard_state.set_commit(sqlite_hard_state.commit as u64);
        hard_state
    }
}

impl Default for SqliteHardState {
    fn default() -> Self {
        let hard_state = HardState::new();

        hard_state.into()
    }
}

