use std::panic;
use std::panic::AssertUnwindSafe;
use std::panic::UnwindSafe;
use std::path::Path;
use tempfile;
use tempfile::TempDir;

fn run_test<S, T, D, TParam, DParam, TRet>(
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

pub fn test_with_temp_dir<S, T, TParam, TRet>(
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

fn create_temp_dir() -> TempDir {
    tempfile::tempdir().unwrap()
}
