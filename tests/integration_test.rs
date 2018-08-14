extern crate rusqlite;
extern crate sqlite_commands;
extern crate tempfile;

use rusqlite::Connection;
use rusqlite::types::ToSql;
use sqlite_commands::connection::AccessConnection;
use sqlite_commands::connection::ReadOnly;
use sqlite_commands::Query;
use sqlite_commands::QueryResultRow;
use common::temp_db::with_test_db_connections;
use std::panic::AssertUnwindSafe;
use sqlite_commands::Execute;
use sqlite_commands::connection::ReadWrite;

mod common;

#[test]
fn test_query_indexed() {
    fn test_query_indexed_parameters(sql: &str, queued_params: AssertUnwindSafe<&[&[&(ToSql)]]>) {
        with_test_db_connections(ReadOnly, |mut test_conn: AccessConnection<ReadOnly>, expected_conn: Connection| {
            let queued_params = queued_params.0;

            let query = Query::new_indexed(&sql, queued_params).unwrap();

            let query_results = query.apply_to_conn(&mut test_conn).unwrap();

            let mapped_query_results: Vec<Vec<_>> = query_results.into_iter().map(|query_result| {
                query_result.as_slice().iter().map(|row: &QueryResultRow| {
                    let rank: i32 = row.get(0);
                    let name: String = row.get(1);
                    let alpha_2: String = row.get(2);
                    let alpha_3: String = row.get(3);
                    (rank, name, alpha_2, alpha_3, )
                }).collect()
            }).collect();

            let mut expected_stmt = expected_conn.prepare(&sql).unwrap();

            let expected_results = queued_params.iter().map(|params| {
                let mapped_rows = expected_stmt.query_map(params, |row| {
                    let rank: i32 = row.get(0);
                    let name: String = row.get(1);
                    let alpha_2: String = row.get(2);
                    let alpha_3: String = row.get(3);
                    (rank, name, alpha_2, alpha_3, )
                }).unwrap();

                mapped_rows.map(|row| row.unwrap()).collect::<Vec<_>>()
            }).collect::<Vec<_>>();

            assert_eq!(expected_results, mapped_query_results);
        });
    }

    let no_param = include_str!("res/sql/test_query_no_param.sql");
    let indexed_param = include_str!("res/sql/test_query_indexed_param.sql");
    let indexed_params = include_str!("res/sql/test_query_indexed_params.sql");

    let test_cases: Vec<(&str, Vec<Vec<&ToSql>>)> = vec![
        (no_param, vec![vec![]]),
        (indexed_param, vec![vec![&"cn"]]),
        (indexed_param, vec![vec![&"cn"], vec![&"j_"]]),
        (indexed_params, vec![vec![&"a_", &10], vec![&"b_", &60]]),
    ];

    for (sql, queued_params) in test_cases {
        let queued_params_slices: Vec<_> = queued_params.iter().map(|vec| vec.as_slice()).collect();

        test_query_indexed_parameters(sql, AssertUnwindSafe(&queued_params_slices));
    }
}

#[test]
fn test_execute_indexed() {
    fn test_execute_indexed_parameters(sql: &str, queued_params: AssertUnwindSafe<&[&[&(ToSql)]]>) {
        with_test_db_connections(ReadWrite, |mut test_conn: AccessConnection<ReadWrite>, expected_conn: Connection| {
            let queued_params = queued_params.0;

            let execute = Execute::new_indexed(&sql, queued_params).unwrap();

            let query_results = execute.apply_to_conn(&mut test_conn).unwrap();

            let mapped_query_results: Vec<_> = query_results.into_iter().map(|query_result| query_result.changes()).collect();

            let mut expected_stmt = expected_conn.prepare(&sql).unwrap();

            let expected_results = queued_params.iter().map(|params| {
                expected_stmt.execute(params).unwrap()
            }).collect::<Vec<_>>();

            assert_eq!(expected_results, mapped_query_results);
        });
    }

    let no_param = include_str!("res/sql/test_execute_no_param.sql");
    let indexed_param = include_str!("res/sql/test_execute_indexed_param.sql");
    let indexed_params = include_str!("res/sql/test_execute_indexed_params.sql");

    let test_cases: Vec<(&str, Vec<Vec<&ToSql>>)> = vec![
        (no_param, vec![vec![]]),
        (indexed_param, vec![vec![&"cn"]]),
        (indexed_param, vec![vec![&"cn"], vec![&"j_"]]),
        (indexed_params, vec![vec![&"a_", &10], vec![&"b_", &60]]),
    ];

    for (sql, queued_params) in test_cases {
        let queued_params_slices: Vec<_> = queued_params.iter().map(|vec| vec.as_slice()).collect();

        test_execute_indexed_parameters(sql, AssertUnwindSafe(&queued_params_slices));
    }
}

// TODO: test_query_named
// TODO: test_execute_named

// TODO: test_bulk_query
// TODO: test_bulk_execute

// Negative tests:

// TODO: test_query_err
// TODO: test_execute_err

// TODO: test_bulk_query_err (middle of transaction)
// TODO: test_bulk_execute_err (middle of transaction)
