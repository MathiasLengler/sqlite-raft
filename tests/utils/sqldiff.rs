use std::path::Path;
use std::process::Command;
use std::process::Output;


pub fn assert_db_eq(test_db_path: &Path, expected_db_path: &Path) {
    let output = exec_sqldiff(test_db_path, expected_db_path);

    assert_eq!(output.stdout.len(), 0, "Unexpected sqldiff stdout content: {:?}", output);
    assert_eq!(output.stderr.len(), 0, "Unexpected sqldiff stderr content: {:?}", output);
}

fn exec_sqldiff(test_db_path: &Path, expected_db_path: &Path) -> Output {
    Command::new("sqldiff").args(&[test_db_path, expected_db_path]).output().unwrap()
}

