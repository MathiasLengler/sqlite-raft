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
        eprintln!("test_db_path = {:?}", test_db_path);

        let conn = Connection::open(&test_db_path).unwrap();

        let test_conn = AccessConnection::open(ReadOnly, &test_db_path).unwrap();

        let sql = r#"SELECT * from countrys WHERE countries.code == "CN""#;

        let params : &[&[&ToSql]] = &[&[]];

        let query_result = Query::new_indexed(&sql, params).unwrap();

        eprintln!("query_result = {:?}", query_result);
    });
}
