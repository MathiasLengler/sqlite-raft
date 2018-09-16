use std::path::Path;
use std::process::Command;
use std::process::Output;


pub fn assert_db_eq(test_db_path: &Path, expected_db_path: &Path) {
    let output = exec_sqldiff(test_db_path, expected_db_path);

    assert!(output.stdout.is_empty(), "Unexpected sqldiff stdout content: {:?}", output);
    assert!(output.stderr.is_empty(), "Unexpected sqldiff stderr content: {:?}", output);
}

fn exec_sqldiff(test_db_path: &Path, expected_db_path: &Path) -> Output {
    Command::new("sqldiff").args(&[test_db_path, expected_db_path]).output().unwrap()
}

