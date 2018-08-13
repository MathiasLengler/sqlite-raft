//! Shell:
//! sqldiff .\expected.sqlite3 .\test.sqlite3
//! INSERT INTO countries(rowid,code,name,population) VALUES(10,'ID','Indonesia',357);
//!
//! Output { status: ExitStatus(ExitStatus(0)), stdout: "DELETE FROM countries WHERE rowid=10;\r\n", stderr: "" }

// TODO: wrapper around sqldiff to assert that two dbs are equal

use std::path::Path;
use std::process::Command;
use std::process::Output;


pub fn assert_db_eq(test_db_path: &Path, expected_db_path: &Path) {
    let mut output = exec_sqldiff(test_db_path, expected_db_path);

    output.stdout.extend(b"foo");

    assert_eq!(output.stdout.len(), 0, "Unexpected sqldiff stdout content: {:?}", output);
    assert_eq!(output.stderr.len(), 0, "Unexpected sqldiff stderr content: {:?}", output);
}

fn exec_sqldiff(test_db_path: &Path, expected_db_path: &Path) -> Output {
    Command::new("sqldiff").args(&[test_db_path, expected_db_path]).output().unwrap()
}

