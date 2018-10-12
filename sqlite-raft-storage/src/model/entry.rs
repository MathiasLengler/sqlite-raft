use error::index::BoundViolation;
use error::index::InvalidEntryIndex;
use error::Result;
use failure::Backtrace;
use model::core::CoreId;
use protobuf::ProtobufEnum;
use raft::eraftpb::Entry;
use raft::eraftpb::EntryType;
use raft::Error as RaftError;
use raft::StorageError as RaftStorageError;
use rusqlite::Result as RusqliteResult;
use rusqlite::Row;
use rusqlite::Transaction;
use rusqlite::types::ToSql;

// TODO: move Entries and Entry to modules
// TODO: debug_assert entries ascending sequence with no gaps

#[derive(Debug)]
pub struct SqliteEntries {
    entries: Vec<SqliteEntry>
}

impl SqliteEntries {
    const SQL_QUERY_RANGE: &'static str =
        include_str!("../../res/sql/entry/query_range.sql");

    pub fn replace_all(&self, tx: &Transaction, core_id: CoreId) -> Result<()> {
        SqliteEntry::delete_all(&tx, core_id)?;

        self.insert(&tx, core_id)?;

        Ok(())
    }

    pub fn insert(&self, tx: &Transaction, core_id: CoreId) -> Result<()> {
        for entry in &self.entries {
            entry.insert(&tx, core_id)?;
        }
        Ok(())
    }

    pub fn append(&self, tx: &Transaction, core_id: CoreId) -> Result<()> {
        if self.entries.is_empty() {
            return Ok(());
        }

        let first_entry = self.entries.first().unwrap();
        first_entry.truncate_right(tx, core_id)?;

        self.insert(tx, core_id)
    }

    pub fn query(tx: &Transaction, core_id: CoreId, low: u64, high: u64) -> Result<SqliteEntries> {
        let high_inclusive = high - 1;

        Self::validate_index_range(
            // Don't return first dummy entry
            low - 1,
            high_inclusive,
            SqliteEntry::first_index(&tx, core_id)?,
            SqliteEntry::last_index(&tx, core_id)?,
        )?;


        Self::query_inclusive_range(tx, core_id, low, high_inclusive)
    }

    fn query_inclusive_range(tx: &Transaction, core_id: CoreId, low: u64, high_inclusive: u64) -> Result<SqliteEntries> {
        let low = low as i64;
        let high_inclusive = high_inclusive as i64;
        let mut stmt = tx.prepare(Self::SQL_QUERY_RANGE)?;
        let rows = stmt.query_map_named(
            &Self::query_parameters(&low, &high_inclusive, &core_id),
            SqliteEntry::from_row,
        )?;

        Ok(SqliteEntries {
            entries: rows.collect::<RusqliteResult<Vec<_>>>()?,
        })
    }

    pub fn query_all(tx: &Transaction, core_id: CoreId) -> Result<SqliteEntries> {
        Self::query_inclusive_range(tx, core_id,
                                    SqliteEntry::first_index(&tx, core_id)?,
                                    SqliteEntry::last_index(&tx, core_id)?)
    }

    fn query_parameters<'a>(low: &'a i64, high_inclusive: &'a i64, core_id: &'a CoreId) -> [(&'static str, &'a ToSql); 3] {
        [
            (":low", low),
            (":high_inclusive", high_inclusive),
            core_id.as_named_param(),
        ]
    }

    // TODO: unit test (check inclusive)
    fn validate_index_range(low: u64, high_inclusive: u64, first_index: u64, last_index: u64) -> Result<()> {
        SqliteEntry::validate_index(low, first_index, last_index)?;
        SqliteEntry::validate_index(high_inclusive, first_index, last_index)?;

        // only contains dummy entries.
        if first_index == last_index {
            return Err(RaftError::Store(RaftStorageError::Unavailable).into());
        }

        Ok(())
    }
}

impl Default for SqliteEntries {
    fn default() -> Self {
        SqliteEntries {
            entries: vec![SqliteEntry::default()],
        }
    }
}

impl From<Vec<Entry>> for SqliteEntries {
    fn from(entries: Vec<Entry>) -> Self {
        SqliteEntries {
            entries: entries.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<SqliteEntries> for Vec<Entry> {
    fn from(sqlite_entries: SqliteEntries) -> Self {
        sqlite_entries.entries.into_iter().map(Into::into).collect()
    }
}

#[derive(Debug)]
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
        include_str!("../../res/sql/entry/delete.sql");
    const SQL_DELETE_GREATER_OR_EQUAL_INDEX: &'static str =
        include_str!("../../res/sql/entry/delete_greater_or_equal_index.sql");
    const SQL_QUERY: &'static str =
        include_str!("../../res/sql/entry/query.sql");
    const SQL_QUERY_FIRST_INDEX: &'static str =
        include_str!("../../res/sql/entry/query_first_index.sql");
    const SQL_QUERY_LAST_INDEX: &'static str =
        include_str!("../../res/sql/entry/query_last_index.sql");
    const SQL_INSERT: &'static str =
        include_str!("../../res/sql/entry/insert.sql");

    fn as_named_params<'a>(&'a self, core_id: &'a CoreId) -> [(&'static str, &'a ToSql); 7] {
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

    fn from_row(row: &Row) -> SqliteEntry {
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

    fn validate_index(idx: u64, first_index: u64, last_index: u64) -> Result<()> {
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

    pub fn first_index(tx: &Transaction, core_id: CoreId) -> Result<u64> {
        let index = tx.query_row_named(
            Self::SQL_QUERY_FIRST_INDEX,
            &[core_id.as_named_param()],
            Self::index_from_row,
        )?;
        Ok(index as u64)
    }

    pub fn last_index(tx: &Transaction, core_id: CoreId) -> Result<u64> {
        let index = tx.query_row_named(
            Self::SQL_QUERY_LAST_INDEX,
            &[core_id.as_named_param()],
            Self::index_from_row,
        )?;
        Ok(index as u64)
    }

    pub fn query(tx: &Transaction, core_id: CoreId, idx: u64) -> Result<SqliteEntry> {
        Self::validate_index(
            idx,
            SqliteEntry::first_index(&tx, core_id)?,
            SqliteEntry::last_index(&tx, core_id)?,
        )?;

        let idx = idx as i64;

        let sqlite_entry = tx.query_row_named(
            Self::SQL_QUERY,
            &Self::query_params(&idx, &core_id),
            Self::from_row,
        )?;

        Ok(sqlite_entry)
    }

    fn query_params<'a>(idx: &'a i64, core_id: &'a CoreId) -> [(&'static str, &'a ToSql); 2] {
        [
            (":index", idx),
            core_id.as_named_param(),
        ]
    }

    pub fn insert(&self, tx: &Transaction, core_id: CoreId) -> Result<()> {
        tx.execute_named(Self::SQL_INSERT, &self.as_named_params(&core_id))?;
        Ok(())
    }

    pub fn delete_all(tx: &Transaction, core_id: CoreId) -> Result<()> {
        tx.execute_named(Self::SQL_DELETE, &[core_id.as_named_param()])?;
        Ok(())
    }

    /// Truncate the log so this entry can be inserted at the end of the log.
    ///
    /// In other words: delete all entries with an index greater or equal to this entry
    fn truncate_right(&self, tx: &Transaction, core_id: CoreId) -> Result<()> {
        tx.execute_named(Self::SQL_DELETE_GREATER_OR_EQUAL_INDEX, &Self::query_params(&self.index, &core_id))?;
        Ok(())
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
    use super::*;
    use error::Error;

    #[test]
    fn test_validate_index() {
        let tests = vec![
            ((2, 3, 5), Err(Error::from(InvalidEntryIndex {
                kind: BoundViolation::TooSmall,
                ..Default::default()
            }))),
            ((3, 3, 5), Ok(())),
            ((4, 3, 5), Ok(())),
            ((5, 3, 5), Ok(())),
            ((6, 3, 5), Err(Error::from(InvalidEntryIndex {
                kind: BoundViolation::TooLarge,
                ..Default::default()
            }))),
        ];

        for (i, ((idx, first_index, last_index), exp_res)) in tests.into_iter().enumerate() {
            let res = SqliteEntry::validate_index(idx, first_index, last_index);

            if res != exp_res {
                panic!("#{}: expect res {:?}, got {:?}.", i, exp_res, res);
            }
        }
    }

    #[test]
    fn test_validate_index_range() {
        use error::index::BoundViolation::*;

        let tests = vec![
            ((3, 3, 3, 3), Err(RaftError::Store(RaftStorageError::Unavailable).into())),
            // low = 2
            ((2, 2, 3, 5), Err(Error::from(InvalidEntryIndex::from(TooSmall)))),
            ((2, 3, 3, 5), Err(Error::from(InvalidEntryIndex::from(TooSmall)))),
            ((2, 5, 3, 5), Err(Error::from(InvalidEntryIndex::from(TooSmall)))),
            ((2, 6, 3, 5), Err(Error::from(InvalidEntryIndex::from(TooSmall)))),
            // low = 3
            ((3, 2, 3, 5), Err(Error::from(InvalidEntryIndex::from(TooSmall)))),
            ((3, 3, 3, 5), Ok(())),
            ((3, 5, 3, 5), Ok(())),
            ((3, 6, 3, 5), Err(Error::from(InvalidEntryIndex::from(TooLarge)))),
            // low = 5
            ((5, 2, 3, 5), Err(Error::from(InvalidEntryIndex::from(TooSmall)))),
            ((5, 3, 3, 5), Ok(())),
            ((5, 5, 3, 5), Ok(())),
            ((5, 6, 3, 5), Err(Error::from(InvalidEntryIndex::from(TooLarge)))),
            // low = 6
            ((6, 2, 3, 5), Err(Error::from(InvalidEntryIndex::from(TooLarge)))),
            ((6, 3, 3, 5), Err(Error::from(InvalidEntryIndex::from(TooLarge)))),
            ((6, 5, 3, 5), Err(Error::from(InvalidEntryIndex::from(TooLarge)))),
            ((6, 6, 3, 5), Err(Error::from(InvalidEntryIndex::from(TooLarge)))),
        ];

        for (i, ((low, high_inclusive, first_index, last_index), exp_res)) in tests.into_iter().enumerate() {
            let res = SqliteEntries::validate_index_range(low, high_inclusive, first_index, last_index);

            if res != exp_res {
                panic!("#{}: expect res {:?}, got {:?}.", i, exp_res, res);
            }
        }
    }
}
