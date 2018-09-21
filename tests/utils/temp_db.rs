use rusqlite;
use rusqlite::Connection;
use sqlite_requests::connection::access::Access;
use sqlite_requests::connection::AccessConnection;
use sqlite_requests::connection::access::ReadOnly;
use sqlite_requests::connection::access::WriteOnly;
use std::panic;
use std::panic::UnwindSafe;
use std::panic::AssertUnwindSafe;
use std::path::PathBuf;
use tempfile;
use utils::sqldiff::assert_db_eq;

pub fn run_test<S, T, D, TParam, DParam, TRet>(
    setup: S,
    test: T,
    teardown: D,
) -> TRet
    where S: FnOnce() -> ((TParam, DParam)),
          T: FnOnce(TParam) -> TRet,
          D: FnOnce(DParam) -> () + UnwindSafe,
          DParam: UnwindSafe,
{
    let (test_param, teardown_param) = setup();

    let test_result = panic::catch_unwind(AssertUnwindSafe(|| {
        test(test_param)
    }));

    let teardown_result = panic::catch_unwind(|| {
        teardown(teardown_param);
    });

    let test_return = match test_result {
        Ok(test_return) => test_return,
        Err(err) => {
            panic::resume_unwind(err);
        }
    };

    if let Err(err) = teardown_result {
        panic::resume_unwind(err);
    }

    test_return
}

pub fn with_test_db_paths<T>(f: impl FnOnce(PathBuf, PathBuf) -> T) -> T {
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
    })
}

pub fn with_test_dbs<A: Access>(access: A, f: impl FnOnce(AccessConnection<A>, Connection) -> ()) {
    with_test_db_paths(|test_db_path: PathBuf, expected_db_path: PathBuf| {
        f(AccessConnection::open(access, &test_db_path).unwrap(),
          Connection::open(&expected_db_path).unwrap());

        assert_db_eq(&test_db_path, &expected_db_path);
    })
}

pub fn with_single_test_db<T>(f: impl FnOnce(AccessConnection<ReadOnly>, AccessConnection<WriteOnly>) -> T) -> T {
    with_test_db_paths(|test_db_path: PathBuf, _: PathBuf| {
        f(AccessConnection::open(ReadOnly, &test_db_path).unwrap(),
          AccessConnection::open(WriteOnly, &test_db_path).unwrap())
    })
}
