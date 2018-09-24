use error::Result;
use model::core::CoreId;
use protobuf::ProtobufEnum;
use raft::eraftpb::Entry;
use raft::eraftpb::EntryType;
use rusqlite::Row;
use rusqlite::Transaction;
use rusqlite::types::ToSql;

// TODO: move Entries and Entry to modules
// TODO: debug_assert entries ascending sequence with no gaps

pub struct SqliteEntries {
    entries: Vec<SqliteEntry>
}

impl SqliteEntries {
    pub fn insert_or_replace(&self, mut tx: &mut Transaction, core_id: CoreId) -> Result<()> {
        SqliteEntry::delete_all(&mut tx, core_id)?;

        self.insert(&mut tx, core_id)?;

        Ok(())
    }

    pub fn append(&self) -> Result<()> {
        // TODO: reverse implementation from test_storage_append
        // seems to be deletion if entries[0].index < last_index

        unimplemented!()
    }

    pub fn query(low: u64, high: u64, max_size: u64) -> Result<SqliteEntries> {
        // TODO: SQL_QUERY_RANGE
        // TODO: reverse implementation from test_storage_entries
        // TODO: limit to max_size

        unimplemented!()
    }

    fn insert(&self, mut tx: &mut Transaction, core_id: CoreId) -> Result<()> {
        for entry in &self.entries {
            entry.insert(&mut tx, core_id)?;
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

pub struct SqliteEntry {
    index: i64,
    term: i64,
    entry_type: i32,
    data: Vec<u8>,
    context: Vec<u8>,
    sync_log: bool,
}

impl SqliteEntry {
    const SQL_QUERY_RANGE: &'static str =
        include_str!("../../res/sql/entry/query_range.sql");
    const SQL_DELETE: &'static str =
        include_str!("../../res/sql/entry/delete.sql");
    const SQL_QUERY_FIRST_INDEX: &'static str =
        include_str!("../../res/sql/entry/query_first_index.sql");
    const SQL_QUERY_LAST_INDEX: &'static str =
        include_str!("../../res/sql/entry/query_last_index.sql");
    const SQL_INSERT: &'static str =
        include_str!("../../res/sql/entry/insert.sql");

    pub fn as_named_params<'a>(&'a self, core_id: &'a CoreId) -> [(&'static str, &'a ToSql); 7] {
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
        unimplemented!()
    }

    pub fn first_index(mut tx: &mut Transaction, core_id: CoreId) -> i64 {
        unimplemented!()
    }

    pub fn last_index(mut tx: &mut Transaction, core_id: CoreId) -> i64 {
        unimplemented!()
    }

    pub fn query(idx: i64) -> Result<SqliteEntry> {
        // TODO: idx == index ? compare with MemStorage tests
        // TODO: SQL_QUERY_INDEX

        unimplemented!()
    }

    pub fn insert(&self, mut tx: &mut Transaction, core_id: CoreId) -> Result<()> {
        tx.execute_named(Self::SQL_INSERT, &self.as_named_params(&core_id))?;
        Ok(())
    }

    pub fn delete_all(mut tx: &mut Transaction, core_id: CoreId) -> Result<()> {
        tx.execute_named(Self::SQL_DELETE, &[core_id.as_named_param()])?;
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

