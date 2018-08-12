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
fn test_query() {
    common::with_test_db(|test_db_path: PathBuf, expected_db_path: PathBuf| {
        let mut test_conn = AccessConnection::open(ReadOnly, &test_db_path).unwrap();

        let sql = include_str!("res/sql/test_query.sql");

        let params : &[&[&ToSql]] = &[&[]];

        let query = Query::new_indexed(&sql, params).unwrap();

        eprintln!("query = {:?}", query);

        let query_result = query.apply_to_conn(&mut test_conn).unwrap();

        eprintln!("query_result = {:?}", query_result);

        let mut expected_conn = Connection::open(&expected_db_path).unwrap();
        expected_conn.query_row(&sql, params, unimplemented!())


        // TODO: assert query result equal
        // TODO: assert dbs equal
    });
}
