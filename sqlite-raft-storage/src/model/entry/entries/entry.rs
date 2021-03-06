use error::Error;
use error::index::BoundViolation;
use error::index::InvalidEntryIndex;
use error::Result;
use failure::Backtrace;
use model::core::CoreId;
use model::core::CoreTx;
use protobuf::ProtobufEnum;
use raft::eraftpb::Entry;
use raft::eraftpb::EntryType;
use rusqlite::Error as RusqliteError;
use rusqlite::Row;
use rusqlite::types::ToSql;


#[derive(Debug, Clone)]
pub struct SqliteEntry {
    index: i64,
    term: i64,
    entry_type: i32,
    data: Vec<u8>,
    context: Vec<u8>,
    sync_log: bool,
}

impl SqliteEntry {
    const SQL_DELETE: &'static str =
        include_str!("../../../../res/sql/entry/delete_all.sql");
    const SQL_DELETE_GREATER_OR_EQUAL_INDEX: &'static str =
        include_str!("../../../../res/sql/entry/delete_greater_or_equal_index.sql");
    const SQL_DELETE_SMALLER_INDEX: &'static str =
        include_str!("../../../../res/sql/entry/delete_smaller_index.sql");
    const SQL_QUERY: &'static str =
        include_str!("../../../../res/sql/entry/query.sql");
    const SQL_QUERY_FIRST_INDEX: &'static str =
        include_str!("../../../../res/sql/entry/query_first_index.sql");
    const SQL_QUERY_LAST_INDEX: &'static str =
        include_str!("../../../../res/sql/entry/query_last_index.sql");
    const SQL_INSERT: &'static str =
        include_str!("../../../../res/sql/entry/insert.sql");

    fn assert_invariants(core_tx: &CoreTx) -> Result<()> {
        use super::SqliteEntries;

        if Self::is_not_empty(core_tx)? {
            SqliteEntries::query_all(core_tx)?;
        }

        Ok(())
    }

    // TODO: replace hacky implementation
    pub(super) fn is_not_empty(core_tx: &CoreTx) -> Result<bool> {
        Ok(match SqliteEntry::last_index(core_tx) {
            Ok(_) => {
                true
            }
            Err(Error::Rusqlite(RusqliteError::QueryReturnedNoRows, _)) => {
                false
            }
            Err(other_err) => Err(other_err)?
        })
    }

    fn as_named_params<'a>(&'a self, core_id: &'a CoreId) -> [(&'static str, &'a dyn ToSql); 7] {
        [
            (":index", &self.index),
            (":term", &self.term),
            (":entry_type", &self.entry_type),
            (":data", &self.data),
            (":context", &self.context),
            (":sync_log", &self.sync_log),
            core_id.as_named_param(),
        ]
    }

    pub(super) fn from_row(row: &Row) -> SqliteEntry {
        SqliteEntry {
            index: row.get("index"),
            term: row.get("term"),
            entry_type: row.get("entry_type"),
            data: row.get("data"),
            context: row.get("context"),
            sync_log: row.get("sync_log"),
        }
    }

    fn index_from_row(row: &Row) -> i64 {
        row.get("index")
    }

    pub(super) fn validate_index(idx: u64, first_index: u64, last_index: u64) -> Result<()> {
        if idx < first_index {
            return Err(InvalidEntryIndex {
                kind: BoundViolation::TooSmall,
                first_index,
                last_index,
                invalid_index: idx,
                backtrace: Backtrace::new(),
            }.into());
        }

        if idx > last_index {
            return Err(InvalidEntryIndex {
                kind: BoundViolation::TooLarge,
                first_index,
                last_index,
                invalid_index: idx,
                backtrace: Backtrace::new(),
            }.into());
        }

        Ok(())
    }

    pub fn first_index(core_tx: &CoreTx) -> Result<u64> {
        let index = core_tx.tx().query_row_named(
            Self::SQL_QUERY_FIRST_INDEX,
            &[core_tx.core_id().as_named_param()],
            Self::index_from_row,
        )?;
        Ok(index as u64)
    }

    pub fn last_index(core_tx: &CoreTx) -> Result<u64> {
        let index = core_tx.tx().query_row_named(
            Self::SQL_QUERY_LAST_INDEX,
            &[core_tx.core_id().as_named_param()],
            Self::index_from_row,
        )?;
        Ok(index as u64)
    }

    pub fn query(core_tx: &CoreTx, idx: u64) -> Result<SqliteEntry> {
        Self::validate_index(
            idx,
            SqliteEntry::first_index(&core_tx)?,
            SqliteEntry::last_index(&core_tx)?,
        )?;

        let idx = idx as i64;

        let sqlite_entry = core_tx.tx().query_row_named(
            Self::SQL_QUERY,
            &Self::query_params(&idx, &core_tx.core_id()),
            Self::from_row,
        )?;

        Ok(sqlite_entry)
    }

    fn query_params<'a>(idx: &'a i64, core_id: &'a CoreId) -> [(&'static str, &'a dyn ToSql); 2] {
        [
            (":index", idx),
            core_id.as_named_param(),
        ]
    }

    pub(super) fn insert(&self, core_tx: &CoreTx) -> Result<()> {
        debug_assert_eq!(Self::assert_invariants(core_tx), Ok(()), "SqliteEntry::insert precondition failed");

        core_tx.tx().execute_named(Self::SQL_INSERT, &self.as_named_params(&core_tx.core_id()))?;

        debug_assert_eq!(Self::assert_invariants(core_tx), Ok(()), "SqliteEntry::insert postcondition failed");

        Ok(())
    }

    pub(super) fn delete_all(core_tx: &CoreTx) -> Result<()> {
        core_tx.tx().execute_named(Self::SQL_DELETE, &[core_tx.core_id().as_named_param()])?;
        Ok(())
    }

    /// Truncate the log so this entry can be inserted at the end of the log.
    ///
    /// In other words: delete all entries with an index greater or equal than this entry.
    pub(super) fn truncate_right(&self, core_tx: &CoreTx) -> Result<()> {
        core_tx.tx().execute_named(Self::SQL_DELETE_GREATER_OR_EQUAL_INDEX, &Self::query_params(&self.index, &core_tx.core_id()))?;
        Ok(())
    }

    /// Truncate the log so this entry would be the first entry in the log.
    ///
    /// In other words: delete all entries with an index smaller than this entry.
    pub fn truncate_left(&self, core_tx: &CoreTx) -> Result<()> {
        core_tx.tx().execute_named(Self::SQL_DELETE_SMALLER_INDEX, &Self::query_params(&self.index, &core_tx.core_id()))?;
        Ok(())
    }

    pub(super) fn try_sequence(&self, previous_entry: &Self) -> Result<()> {
        use error::entries::NonSequentialEntryPair;
        use error::entries::SequenceViolation::*;

        if self.index != previous_entry.index + 1 {
            Err(NonSequentialEntryPair {
                incompatible_entry: self.clone().into(),
                previous_entry: previous_entry.clone().into(),
                cause: IncompatibleIndex,
                backtrace: Backtrace::new(),
            }.into())
        } else if self.term < previous_entry.term {
            Err(NonSequentialEntryPair {
                incompatible_entry: self.clone().into(),
                previous_entry: previous_entry.clone().into(),
                cause: DecreasingTerm,
                backtrace: Backtrace::new(),
            }.into())
        } else {
            Ok(())
        }
    }
}

impl From<Entry> for SqliteEntry {
    fn from(mut entry: Entry) -> Self {
        SqliteEntry {
            index: entry.get_index() as i64,
            term: entry.get_term() as i64,
            entry_type: entry.get_entry_type().value(),
            data: entry.take_data(),
            context: entry.take_context(),
            sync_log: entry.get_sync_log(),
        }
    }
}

impl From<SqliteEntry> for Entry {
    fn from(sqlite_entry: SqliteEntry) -> Self {
        let mut entry = Entry::new();
        entry.set_index(sqlite_entry.index as u64);
        entry.set_term(sqlite_entry.term as u64);
        entry.set_entry_type(EntryType::from_i32(sqlite_entry.entry_type).unwrap());
        entry.set_data(sqlite_entry.data);
        entry.set_context(sqlite_entry.context);
        entry.set_sync_log(sqlite_entry.sync_log);
        entry
    }
}

impl Default for SqliteEntry {
    fn default() -> Self {
        let entry = Entry::new();

        entry.into()
    }
}

#[cfg(test)]
mod tests {
    use error::entries::NonSequentialEntryPair;
    use error::entries::SequenceViolation;
    use error::Error;
    use super::*;

    // TODO extract these duplicated utility functions for tests
    fn new_entry(index: i64, term: i64) -> SqliteEntry {
        SqliteEntry {
            index,
            term,
            ..Default::default()
        }
    }

    #[test]
    fn test_validate_index() {
        let tests = vec![
            ((2, 3, 5), Err(Error::from(InvalidEntryIndex::from(BoundViolation::TooSmall)))),
            ((3, 3, 5), Ok(())),
            ((4, 3, 5), Ok(())),
            ((5, 3, 5), Ok(())),
            ((6, 3, 5), Err(Error::from(InvalidEntryIndex::from(BoundViolation::TooLarge)))),
        ];

        for (i, ((idx, first_index, last_index), exp_res)) in tests.into_iter().enumerate() {
            let res = SqliteEntry::validate_index(idx, first_index, last_index);

            if res != exp_res {
                panic!("#{}: expect res {:?}, got {:?}.", i, exp_res, res);
            }
        }
    }

    #[test]
    fn test_try_sequence() {
        let tests = vec![
            (new_entry(3, 3), new_entry(4, 3), Ok(())),
            (new_entry(3, 3), new_entry(4, 4), Ok(())),
            (new_entry(3, 3), new_entry(4, 5), Ok(())),
            // IncompatibleIndex
            (new_entry(3, 3), new_entry(2, 3), Err(Error::from(NonSequentialEntryPair {
                incompatible_entry: new_entry(2, 3).into(),
                previous_entry: new_entry(3, 3).into(),
                cause: SequenceViolation::IncompatibleIndex,
                backtrace: Backtrace::new(),
            }))),
            (new_entry(3, 3), new_entry(3, 3), Err(Error::from(NonSequentialEntryPair {
                incompatible_entry: new_entry(3, 3).into(),
                previous_entry: new_entry(3, 3).into(),
                cause: SequenceViolation::IncompatibleIndex,
                backtrace: Backtrace::new(),
            }))),
            (new_entry(3, 3), new_entry(5, 3), Err(Error::from(NonSequentialEntryPair {
                incompatible_entry: new_entry(5, 3).into(),
                previous_entry: new_entry(3, 3).into(),
                cause: SequenceViolation::IncompatibleIndex,
                backtrace: Backtrace::new(),
            }))),
            // DecreasingTerm
            (new_entry(3, 3), new_entry(4, 2), Err(Error::from(NonSequentialEntryPair {
                incompatible_entry: new_entry(4, 2).into(),
                previous_entry: new_entry(3, 3).into(),
                cause: SequenceViolation::DecreasingTerm,
                backtrace: Backtrace::new(),
            }))),
        ];

        for (i, (previous_entry, new_entry, exp_res)) in tests.into_iter().enumerate() {
            let res = new_entry.try_sequence(&previous_entry);

            if res != exp_res {
                panic!("#{}: expect res {:?}, got {:?}.", i, exp_res, res);
            }
        }
    }
}
