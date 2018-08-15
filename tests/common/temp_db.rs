use rusqlite;
use rusqlite::Connection;
use std::panic;
use std::path::PathBuf;
use tempfile;
use sqlite_commands::connection::Access;
use sqlite_commands::connection::AccessConnection;
use common;

pub fn run_test<S, T, D, TParam, DParam>(
    setup: S,
    test: T,
    teardown: D,
) -> ()
    where S: FnOnce() -> ((TParam, DParam)),
          T: FnOnce(TParam) -> () + panic::UnwindSafe,
          D: FnOnce(DParam) -> () + panic::UnwindSafe,
          TParam: panic::UnwindSafe,
          DParam: panic::UnwindSafe,
{
    let (test_param, teardown_param) = setup();

    let test_result = panic::catch_unwind(|| {
        test(test_param)
    });

    let teardown_result = panic::catch_unwind(|| {
        teardown(teardown_param);
    });

    if let Err(err) = test_result {
        panic::resume_unwind(err);
    }

    if let Err(err) = teardown_result {
        panic::resume_unwind(err);
    }
}

pub fn with_test_db_paths(f: impl FnOnce(PathBuf, PathBuf) -> () + panic::UnwindSafe) {
    run_test(|| {
        let temp_dir_root: PathBuf = [env!("CARGO_MANIFEST_DIR"), "tests", "res", "temp"].iter().collect();

        let temp_dir = tempfile::tempdir_in(&temp_dir_root).unwrap();

        let test_db_path: PathBuf = [temp_dir.path().to_path_buf(), "test.sqlite3".into()].iter().collect();
        let expected_db_path: PathBuf = [temp_dir.path().to_path_buf(), "expected.sqlite3".into()].iter().collect();

        let init_test_db_sql = include_str!("../res/sql/init_test_db.sql");

        let test_db_conn = rusqlite::Connection::open(&test_db_path).unwrap();
        test_db_conn.execute_batch(init_test_db_sql).unwrap();

        let expected_conn = Connection::open(&expected_db_path).unwrap();
        expected_conn.execute_batch(init_test_db_sql).unwrap();

        ((test_db_path, expected_db_path), temp_dir)
    }, |(test_db_path, expected_db_path)| {
        f(test_db_path, expected_db_path)
    }, |temp_dir| {
        temp_dir.close().unwrap();
    });
}

pub fn with_test_db_connections<A: Access + panic::RefUnwindSafe>(
    access: A,
    f: impl FnOnce(AccessConnection<A>, Connection, ) -> () + panic::UnwindSafe) {
    with_test_db_paths(|test_db_path: PathBuf, expected_db_path: PathBuf| {
        f(AccessConnection::open(access, &test_db_path).unwrap(), Connection::open(&expected_db_path).unwrap());

        common::sqldiff::assert_db_eq(&test_db_path, &expected_db_path);
    })
}
