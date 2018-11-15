use utils::with_test_db_path;
use std::path::PathBuf;
use rusqlite::Connection;
use sqlite_user_storage::view::nested_savepoint::NestedSavepoint;
use sqlite_requests::{
    query::Query,
    execute::Execute,
    request::{SqliteRequest, SqliteResponse},
    connection::AccessConnection,
};
use sqlite_requests::connection::access::ReadWrite;

// TODO: fix execute result caching in sqlite-requests

fn request_test_cases() -> Vec<SqliteRequest> {
    vec![
        Execute::new_indexed("CREATE TABLE Test (value INTEGER NOT NULL UNIQUE)", &[&[]]).unwrap().into(),
        Execute::new_indexed("INSERT INTO Test VALUES (13)", &[&[]]).unwrap().into(),
        Execute::new_indexed("CREATE TABLE Test2 (value INTEGER NOT NULL UNIQUE)", &[&[]]).unwrap().into(),
        Query::new_indexed("SELECT * FROM Test", &[&[]]).unwrap().into(),
        Execute::new_indexed("INSERT INTO Test VALUES (42)", &[&[]]).unwrap().into(),
        Query::new_indexed("SELECT * FROM Test", &[&[]]).unwrap().into(),
    ]
}

fn response_test_cases() -> Vec<SqliteResponse> {
    with_test_db_path(|test_db_path: PathBuf| {
        let mut access_connection = AccessConnection::open(ReadWrite, test_db_path).unwrap();

        request_test_cases().iter().map(|sqlite_request| {
            access_connection.run(sqlite_request).unwrap()
        }).collect()
    })
}

fn test_cases() -> Vec<(SqliteRequest, SqliteResponse)> {
    request_test_cases().into_iter().zip(response_test_cases()).collect()
}

#[test]
fn test_nested_savepoint_push() {
    with_test_db_path(|test_db_path: PathBuf| {
        let conn = Connection::open(&test_db_path).unwrap();

        let mut nested_sp = NestedSavepoint::new(&conn);

        for (request, expected_response) in test_cases() {
            let actual_response = nested_sp.push(&request).unwrap();

            assert_eq!(actual_response, expected_response);
        }
    })
}

#[test]
fn test_nested_savepoint_rollback_to() {
    for rollback_depth in 0..=3 {
        with_test_db_path(|test_db_path: PathBuf| {
            let conn = Connection::open(&test_db_path).unwrap();

            let mut nested_sp = NestedSavepoint::new(&conn);

            for request in request_test_cases() {
                nested_sp.push(&request).unwrap();
            }

            nested_sp.rollback_to(rollback_depth).unwrap();

            let mut test_cases = test_cases();

            for (request, expected_response) in test_cases.drain((rollback_depth as usize)..) {
                let actual_response = nested_sp.push(&request).unwrap();

                assert_eq!(actual_response, expected_response, "Rollback depth: {}", rollback_depth)
            }
        });
    }
}
