extern crate rusqlite;
extern crate sqlite_commands;
extern crate tempfile;

use rusqlite::Connection;
use rusqlite::types::ToSql;
use sqlite_commands::connection::AccessConnection;
use sqlite_commands::connection::ReadOnly;
use sqlite_commands::Query;
use std::path::PathBuf;

mod common;

#[test]
fn test_query_indexed() {
    common::with_test_db(|test_db_path: PathBuf, expected_db_path: PathBuf| {
        let mut test_conn = AccessConnection::open(ReadOnly, &test_db_path).unwrap();

        let sql = include_str!("res/sql/test_query_indexed.sql");

        let queued_params: &[&[&ToSql]] = &[&[]];

        let query = Query::new_indexed(&sql, queued_params).unwrap();

        eprintln!("query = {:?}", query);

        let query_result = query.apply_to_conn(&mut test_conn).unwrap();

        eprintln!("query_result = {:?}", query_result);

        let mut expected_conn = Connection::open(&expected_db_path).unwrap();
        let mut expected_stmt = expected_conn.prepare(&sql).unwrap();

        // TODO: collect
        let expected_result = queued_params.iter().map(|params| {
            let mapped_rows = expected_stmt.query_map(params, |_| {
                // TODO
                1
            }).unwrap();

            mapped_rows.map(|row| row.unwrap()).collect::<Vec<_>>()
        }).collect::<Vec<_>>();


        // TODO: assert query result equal (using mapped rows on QueryResult)


        common::sqldiff::assert_db_eq(&test_db_path, &expected_db_path);
    });
}

// TODO: test_query_named
// TODO: test_execute_indexed
// TODO: test_execute_named

// TODO: test_bulk_query
// TODO: test_bulk_execute

// TODO: test_query_err
// TODO: test_execute_err

// TODO: test_bulk_query_err (middle of transaction)
// TODO: test_bulk_execute_err (middle of transaction)
