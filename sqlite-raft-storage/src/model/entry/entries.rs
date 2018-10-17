use error::Result;
use model::core::CoreId;
use raft::eraftpb::Entry;
use raft::Error as RaftError;
use raft::StorageError as RaftStorageError;
use rusqlite::Result as RusqliteResult;
use rusqlite::Transaction;
use rusqlite::types::ToSql;
use super::entry::SqliteEntry;

// TODO: move Entries and Entry to modules
// TODO: debug_assert entries ascending sequence with no gaps

#[derive(Debug)]
pub struct SqliteEntries {
    entries: Vec<SqliteEntry>
}

impl SqliteEntries {
    const SQL_QUERY_RANGE: &'static str =
        include_str!("../../../res/sql/entry/query_range.sql");

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
        match self.entries.first() {
            None => Ok(()),
            Some(first_entry) => {
                first_entry.truncate_right(tx, core_id)?;

                self.insert(tx, core_id)
            },
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use error::Error;
    use error::index::InvalidEntryIndex;

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
