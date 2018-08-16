use integration_test::indexed_test_cases;
use integration_test::named_test_cases;
use rusqlite::Connection;
use rusqlite::types::ToSql;
use sqlite_commands::connection::AccessConnection;
use sqlite_commands::connection::ReadWrite;
use sqlite_commands::execute::BulkExecute;
use sqlite_commands::execute::Execute;
use std::panic::AssertUnwindSafe;
use utils::temp_db::with_test_db_connections;

#[test]
fn test_execute_indexed() {
    fn test_execute_indexed_parameters(sql: &str, queued_params: AssertUnwindSafe<&[&[&(ToSql)]]>) {
        with_test_db_connections(ReadWrite, |mut test_conn: AccessConnection<ReadWrite>, expected_conn: Connection| {
            let queued_params = queued_params.0;

            let execute = Execute::new_indexed(&sql, queued_params).unwrap();
            let execute_results = test_conn.run(&execute).unwrap();
            let mapped_execute_results: Vec<_> = execute_results.into_iter().map(|execute_result| execute_result.changes()).collect();

            let mut expected_stmt = expected_conn.prepare(&sql).unwrap();
            let expected_results = queued_params.iter().map(|params| {
                expected_stmt.execute(params).unwrap()
            }).collect::<Vec<_>>();

            assert_eq!(expected_results, mapped_execute_results);
        });
    }

    let no_param =
        include_str!("../res/sql/test_execute_no_param.sql");
    let indexed_param =
        include_str!("../res/sql/test_execute_indexed_param.sql");
    let indexed_params =
        include_str!("../res/sql/test_execute_indexed_params.sql");

    for (sql, queued_params) in indexed_test_cases(no_param, indexed_param, indexed_params) {
        let queued_params_slices: Vec<_> = queued_params.iter().map(|vec| vec.as_slice()).collect();

        test_execute_indexed_parameters(sql, AssertUnwindSafe(&queued_params_slices));
    }
}

#[test]
fn test_execute_named() {
    fn test_execute_named_parameters(sql: &str, queued_params: AssertUnwindSafe<&[&[(&str, &ToSql)]]>) {
        with_test_db_connections(ReadWrite, |mut test_conn: AccessConnection<ReadWrite>, expected_conn: Connection| {
            let queued_params = queued_params.0;

            let execute = Execute::new_named(&sql, queued_params).unwrap();
            let execute_results = test_conn.run(&execute).unwrap();
            let mapped_execute_results: Vec<_> = execute_results.into_iter().map(|execute_result| execute_result.changes()).collect();

            let mut expected_stmt = expected_conn.prepare(&sql).unwrap();
            let expected_results = queued_params.iter().map(|params| {
                expected_stmt.execute_named(params).unwrap()
            }).collect::<Vec<_>>();

            assert_eq!(expected_results, mapped_execute_results);
        });
    }

    let no_param =
        include_str!("../res/sql/test_execute_no_param.sql");
    let named_param =
        include_str!("../res/sql/test_execute_named_param.sql");
    let named_params =
        include_str!("../res/sql/test_execute_named_params.sql");

    for (sql, queued_params) in named_test_cases(no_param, named_param, named_params) {
        let queued_params_slices: Vec<_> = queued_params.iter().map(|vec| vec.as_slice()).collect();

        test_execute_named_parameters(sql, AssertUnwindSafe(&queued_params_slices));
    }
}

#[test]
fn test_bulk_execute() {
    with_test_db_connections(ReadWrite, |mut test_conn: AccessConnection<ReadWrite>, expected_conn: Connection| {
        let no_param =
            include_str!("../res/sql/test_execute_no_param.sql");
        let indexed_param =
            include_str!("../res/sql/test_execute_indexed_param.sql");
        let indexed_params =
            include_str!("../res/sql/test_execute_indexed_params.sql");


        // Test
        let test_queries = indexed_test_cases(no_param, indexed_param, indexed_params).iter().map(|(sql, queued_params)| {
            let queued_params_slices: Vec<_> = queued_params.iter().map(|vec| vec.as_slice()).collect();

            Execute::new_indexed(&sql, &queued_params_slices).unwrap()
        }).collect();

        let bulk_execute = BulkExecute::new(test_queries);

        let bulk_execute_results = test_conn.run(&bulk_execute).unwrap();

        let mapped_bulk_execute_results: Vec<_> = bulk_execute_results.into_iter().map(|execute_results| {
            let mapped_execute_results: Vec<_> = execute_results.into_iter().map(|execute_result| {
                execute_result.changes()
            }).collect();
            mapped_execute_results
        }).collect();


        //Expected
        let bulk_expected_results: Vec<_> = indexed_test_cases(no_param, indexed_param, indexed_params).iter().map(|(sql, queued_params)| {
            let mut expected_stmt = expected_conn.prepare(&sql).unwrap();
            let expected_results = queued_params.iter().map(|params| {
                let changes = expected_stmt.execute(params).unwrap();
                changes
            }).collect::<Vec<_>>();
            expected_results
        }).collect();

        assert_eq!(mapped_bulk_execute_results, bulk_expected_results);
    });
}


// Negative tests:

// TODO: test_execute_err
// TODO: test_bulk_execute_err (middle of transaction)
