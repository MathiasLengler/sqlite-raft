use integration_test::indexed_test_cases;
use integration_test::named_test_cases;
use rusqlite::Connection;
use rusqlite::types::ToSql;
use sqlite_requests::connection::AccessConnection;
use sqlite_requests::connection::access::WriteOnly;
use sqlite_requests::execute::BulkExecute;
use sqlite_requests::execute::Execute;
use utils::temp_db::with_equal_connections;
use integration_test::queued_params_as_arg;

#[test]
fn test_execute_indexed() {
    fn test_execute_indexed_parameters(sql: &str, queued_params: &[&[&(ToSql)]]) {
        with_equal_connections(WriteOnly, |mut test_conn: AccessConnection<WriteOnly>, expected_conn: Connection| {
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
        let queued_params_slices: Vec<_> = queued_params_as_arg(&queued_params);

        test_execute_indexed_parameters(sql, &queued_params_slices);
    }
}

#[test]
fn test_execute_named() {
    fn test_execute_named_parameters(sql: &str, queued_params: &[&[(&str, &ToSql)]]) {
        with_equal_connections(WriteOnly, |mut test_conn: AccessConnection<WriteOnly>, expected_conn: Connection| {
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
        let queued_params_slices: Vec<_> = queued_params_as_arg(&queued_params);;

        test_execute_named_parameters(sql, &queued_params_slices);
    }
}

#[test]
fn test_bulk_execute() {
    with_equal_connections(WriteOnly, |mut test_conn: AccessConnection<WriteOnly>, expected_conn: Connection| {
        let no_param =
            include_str!("../res/sql/test_execute_no_param.sql");
        let indexed_param =
            include_str!("../res/sql/test_execute_indexed_param.sql");
        let indexed_params =
            include_str!("../res/sql/test_execute_indexed_params.sql");


        // Test
        let test_queries = indexed_test_cases(no_param, indexed_param, indexed_params).iter().map(|(sql, queued_params)| {
            let queued_params_slices: Vec<_> = queued_params_as_arg(&queued_params);;

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
