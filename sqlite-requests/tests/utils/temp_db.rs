use rusqlite::Connection;
use sqlite_requests::connection::access::Access;
use sqlite_requests::connection::AccessConnection;
use std::panic;
use std::panic::UnwindSafe;
use std::panic::AssertUnwindSafe;
use std::path::PathBuf;
use tempfile;
use utils::sqldiff::assert_db_eq;
use std::path::Path;
use tempfile::TempDir;

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

fn test_with_temp_dir<S, T, TParam, TRet>(
    setup: S,
    test: T,
) -> TRet
    where S: FnOnce(&Path) -> (TParam),
          T: FnOnce(TParam) -> TRet,
{
    run_test(|| {
        let temp_dir = create_temp_dir();

        let test_param = setup(temp_dir.path());

        (test_param, temp_dir)
    }, |test_param| {
        test(test_param)
    }, |temp_dir| {
        temp_dir.close().unwrap();
    })
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

fn init_test_db(test_db_path: impl AsRef<Path>) {
    let init_test_db_sql = include_str!("../res/sql/init_test_db.sql");

    let test_db_conn = Connection::open(&test_db_path).unwrap();
    test_db_conn.execute_batch(init_test_db_sql).unwrap();
}

fn create_temp_dir() -> TempDir {
    let temp_dir_root: PathBuf = [env!("CARGO_MANIFEST_DIR"), "tests", "res", "temp"].iter().collect();

    tempfile::tempdir_in(&temp_dir_root).unwrap()
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
