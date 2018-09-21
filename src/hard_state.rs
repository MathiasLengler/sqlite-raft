use rusqlite::Transaction;
use error::Result;

struct SqliteHardState {
    term: u64,
    vote: u64,
    commit: u64,
}

impl SqliteHardState {
    const SQL_QUERY: &'static str = include_str!("../res/sql/hard_state/query.sql");

    fn query(mut tx: &mut Transaction) -> Result<SqliteHardState> {
        tx.query_row(SqliteHardState::SQL_QUERY, &[], |row| {
            let term: i64 = row.get("term");
            let vote: i64 = row.get("vote");
            let commit: i64 = row.get("commit");

            let term = term as u64;
            let vote = vote as u64;
            let commit = commit as u64;

            SqliteHardState {
                term,
                vote,
                commit,
            }
        }).map_err(Into::into)
    }
}
