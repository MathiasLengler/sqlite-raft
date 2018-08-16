use rusqlite::types::ToSql;
use sqlite_commands::Query;
use sqlite_commands::error::Error;
use sqlite_commands::Execute;

mod query;
mod execute;

fn indexed_test_cases<'a>(no_param: &'a str,
                          indexed_param: &'a str,
                          indexed_params: &'a str, ) -> Vec<(&'a str, Vec<Vec<&'static ToSql>>)> {
    vec![
        (no_param, vec![vec![]]),
        (indexed_param, vec![vec![&"cn"]]),
        (indexed_param, vec![vec![&"cn"], vec![&"j_"]]),
        (indexed_params, vec![vec![&"a_", &10]]),
        (indexed_params, vec![vec![&"a_", &10], vec![&"b_", &60]]),
    ]
}

fn named_test_cases<'a>(no_param: &'a str,
                        named_param: &'a str,
                        named_params: &'a str, ) -> Vec<(&'a str, Vec<Vec<(&'static str, &'static ToSql)>>)> {
    vec![
        (no_param, vec![vec![]]),
        (named_param, vec![vec![(&":alpha_2", &"cn")]]),
        (named_param, vec![vec![(&":alpha_2", &"cn")], vec![(&":alpha_2", &"j_")]]),
        (named_params, vec![vec![(&":alpha_2", &"a_"), (&":rank", &10)]]),
        (named_params, vec![vec![(&":alpha_2", &"a_"), (&":rank", &10)],
                            vec![(&":alpha_2", &"b_"), (&":rank", &60)]]),
    ]
}


#[test]
fn test_no_queued_parameters_err() {
    use std::mem::discriminant;

    assert_eq!(discriminant(&Query::new_indexed("foo", &[]).unwrap_err()), discriminant(&Error::NoQueuedParameters));
    assert_eq!(discriminant(&Query::new_named("foo", &[]).unwrap_err()), discriminant(&Error::NoQueuedParameters));
    assert_eq!(discriminant(&Execute::new_indexed("foo", &[]).unwrap_err()), discriminant(&Error::NoQueuedParameters));
    assert_eq!(discriminant(&Execute::new_named("foo", &[]).unwrap_err()), discriminant(&Error::NoQueuedParameters));
}



