pub use self::entries::SqliteEntries;
pub use self::entry::SqliteEntry;

mod entry;
mod entries;

// TODO: debug_assert entries ascending sequence with no gaps
