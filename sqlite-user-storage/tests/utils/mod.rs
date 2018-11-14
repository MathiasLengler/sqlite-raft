use sqlite_test_utils::temp_dir::test_with_temp_dir;
use std::path::Path;
use std::path::PathBuf;


fn setup_test_db(temp_dir_path: &Path) -> PathBuf {
    let test_db_path: PathBuf = temp_dir_path.join("test.sqlite3");
    test_db_path
}

pub fn with_test_db_path<T>(f: impl FnOnce(PathBuf) -> T) -> T {
    test_with_temp_dir(setup_test_db, |test_db_path| {
        f(test_db_path)
    })
}
