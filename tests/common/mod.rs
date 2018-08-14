use std::panic;

pub mod sqldiff;
pub mod temp_db;

pub fn run_test<S, T, D, TParam, DParam>(
    setup: S,
    test: T,
    teardown: D,
) -> ()
    where S: FnOnce() -> ((TParam, DParam)),
          T: FnOnce(TParam) -> () + panic::UnwindSafe,
          D: FnOnce(DParam) -> () + panic::UnwindSafe,
          TParam: panic::UnwindSafe,
          DParam: panic::UnwindSafe,
{
    let (test_param, teardown_param) = setup();

    let test_result = panic::catch_unwind(|| {
        test(test_param)
    });

    let teardown_result = panic::catch_unwind(|| {
        teardown(teardown_param);
    });

    if let Err(err) = test_result {
        panic::resume_unwind(err);
    }

    if let Err(err) = teardown_result {
        panic::resume_unwind(err);
    }
}



