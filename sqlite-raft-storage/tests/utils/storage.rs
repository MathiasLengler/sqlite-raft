use sqlite_raft_storage::storage_traits::StorageTestable;
use raft::storage::MemStorage;
use sqlite_raft_storage::SqliteStorage;
use utils::temp_db::with_test_db_path;
use std::path::PathBuf;

pub fn test_storage_impls(mut test: impl FnMut(&mut dyn StorageTestable)) {
    with_test_db_path(|test_db_path: PathBuf| {
        let mut mem_storage = MemStorage::new();
        test(&mut mem_storage);

        let mut sqlite_storage = SqliteStorage::open(test_db_path).unwrap();
        test(&mut sqlite_storage);
    });
}

