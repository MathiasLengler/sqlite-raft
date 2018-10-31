use error::Result;
use model::core::CoreId;
use model::core::CoreTx;
use raft::eraftpb::Entry;
use raft::Error as RaftError;
use raft::StorageError as RaftStorageError;
use rusqlite::Result as RusqliteResult;
use rusqlite::types::ToSql;
use super::SqliteEntry;

pub mod entry;


#[derive(Debug)]
pub struct SqliteEntries {
    entries: Vec<SqliteEntry>
}

impl SqliteEntries {
    const SQL_QUERY_RANGE: &'static str =
        include_str!("../../../../res/sql/entry/query_range.sql");

    pub fn try_from_entry_vec(entries: Vec<Entry>) -> Result<Self> {
        SqliteEntries {
            entries: entries.into_iter().map(Into::into).collect(),
        }.validate()
    }

    fn try_from_sqlite_entry_vec(entries: Vec<SqliteEntry>) -> Result<Self> {
        SqliteEntries {
            entries,
        }.validate()
    }

    fn validate(self) -> Result<Self> {
        match self.entries.first() {
            None => {}
            Some(first_entry) => {
                self.entries.iter().skip(1).try_fold(
                    first_entry,
                    |previous_entry, entry| -> Result<&SqliteEntry> {
                        entry.try_sequence(previous_entry)?;
                        Ok(entry)
                    },
                )?;
            }
        }

        Ok(self)
    }

    pub fn replace_all(&self, core_tx: &CoreTx) -> Result<()> {
        SqliteEntry::delete_all(&core_tx)?;

        self.insert(&core_tx)?;

        Ok(())
    }

    fn insert(&self, core_tx: &CoreTx) -> Result<()> {
        match self.entries.first() {
            None => {}
            Some(first_entry) => {
                if SqliteEntry::is_not_empty(core_tx)? {
                    let last_index = SqliteEntry::last_index(core_tx)?;
                    let current_last_entry = SqliteEntry::query(core_tx, last_index)?;
                    first_entry.try_sequence(&current_last_entry)?;
                }

                for entry in &self.entries {
                    entry.insert(&core_tx)?;
                }
            }
        }

        Ok(())
    }

    pub fn append(&self, core_tx: &CoreTx) -> Result<()> {
        match self.entries.first() {
            None => Ok(()),
            Some(first_entry) => {
                first_entry.truncate_right(core_tx)?;

                self.insert(core_tx)
            }
        }
    }

    pub fn query(core_tx: &CoreTx, low: u64, high: u64) -> Result<SqliteEntries> {
        let high_inclusive = high - 1;

        Self::validate_index_range(
            // Don't return first dummy entry
            low - 1,
            high_inclusive,
            SqliteEntry::first_index(&core_tx)?,
            SqliteEntry::last_index(&core_tx)?,
        )?;


        Self::query_inclusive_range(core_tx, low, high_inclusive)
    }

    fn query_inclusive_range(core_tx: &CoreTx, low: u64, high_inclusive: u64) -> Result<Self> {
        let low = low as i64;
        let high_inclusive = high_inclusive as i64;
        let mut stmt = core_tx.tx().prepare(Self::SQL_QUERY_RANGE)?;
        let rows = stmt.query_map_named(
            &Self::query_parameters(&low, &high_inclusive, &core_tx.core_id()),
            SqliteEntry::from_row,
        )?;

        let entries = rows.collect::<RusqliteResult<Vec<_>>>()?;

        Self::try_from_sqlite_entry_vec(entries)
    }

    pub fn query_all(core_tx: &CoreTx) -> Result<SqliteEntries> {
        Self::query_inclusive_range(core_tx,
                                    SqliteEntry::first_index(&core_tx)?,
                                    SqliteEntry::last_index(&core_tx)?)
    }

    fn query_parameters<'a>(low: &'a i64, high_inclusive: &'a i64, core_id: &'a CoreId) -> [(&'static str, &'a dyn ToSql); 3] {
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

impl From<SqliteEntries> for Vec<Entry> {
    fn from(sqlite_entries: SqliteEntries) -> Self {
        sqlite_entries.entries.into_iter().map(Into::into).collect()
    }
}

#[cfg(test)]
mod tests {
    use error::Error;
    use error::index::InvalidEntryIndex;
    use super::*;

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
