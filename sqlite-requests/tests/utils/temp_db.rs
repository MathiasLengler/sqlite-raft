use rusqlite::Connection;
use sqlite_requests::connection::access::Access;
use sqlite_requests::connection::AccessConnection;
use sqlite_test_utils::sqldiff::assert_db_eq;
use sqlite_test_utils::temp_dir::test_with_temp_dir;
use std::path::Path;
use std::path::PathBuf;


fn init_test_db(test_db_path: impl AsRef<Path>) {
    let init_test_db_sql = include_str!("../res/sql/init_test_db.sql");

    let test_db_conn = Connection::open(&test_db_path).unwrap();
    test_db_conn.execute_batch(init_test_db_sql).unwrap();
}

fn setup_test_dbs(temp_dir_path: &Path) -> (PathBuf, PathBuf) {
    let test_db_path: PathBuf = temp_dir_path.join("test.sqlite3");
    let expected_db_path: PathBuf = temp_dir_path.join("expected.sqlite3");

    init_test_db(&test_db_path);
    init_test_db(&expected_db_path);

    (test_db_path, expected_db_path)
}

fn setup_test_db(temp_dir_path: &Path) -> PathBuf {
    let test_db_path: PathBuf = temp_dir_path.join("test.sqlite3");
    init_test_db(&test_db_path);
    test_db_path
}

fn with_test_db_paths<T>(f: impl FnOnce(PathBuf, PathBuf) -> T) -> T {
    test_with_temp_dir(setup_test_dbs, |(test_db_path, expected_db_path)| {
        f(test_db_path, expected_db_path)
    })
}

fn with_test_db_path<T>(f: impl FnOnce(PathBuf) -> T) -> T {
    test_with_temp_dir(setup_test_db, |test_db_path| {
        f(test_db_path)
    })
}

pub fn with_equal_connections<A: Access>(access: A, f: impl FnOnce(AccessConnection<A>, Connection) -> ()) {
    with_test_db_paths(|test_db_path: PathBuf, expected_db_path: PathBuf| {
        f(AccessConnection::open(access, &test_db_path).unwrap(),
          Connection::open(&expected_db_path).unwrap());

        assert_db_eq(&test_db_path, &expected_db_path);
    })
}

pub fn with_access_connection<T, A: Access>(access: A, f: impl FnOnce(AccessConnection<A>) -> T) -> T {
    with_test_db_path(|test_db_path: PathBuf| {
        f(AccessConnection::open(access, &test_db_path).unwrap())
    })
}
