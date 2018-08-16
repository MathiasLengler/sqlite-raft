use integration_test::indexed_test_cases;
use integration_test::named_test_cases;
use rusqlite::Connection;
use rusqlite::types::ToSql;
use sqlite_commands::connection::AccessConnection;
use sqlite_commands::connection::ReadOnly;
use sqlite_commands::query::BulkQuery;
use sqlite_commands::query::Query;
use std::panic::AssertUnwindSafe;
use utils::query_helper::Country;
use utils::temp_db::with_test_db_connections;


#[test]
fn test_query_indexed() {
    fn test_query_indexed_parameters(sql: &str, queued_params: AssertUnwindSafe<&[&[&(ToSql)]]>) {
        with_test_db_connections(ReadOnly, |mut test_conn: AccessConnection<ReadOnly>, expected_conn: Connection| {
            let queued_params = queued_params.0;

            let query = Query::new_indexed(&sql, queued_params).unwrap();

            let query_results = test_conn.run(&query).unwrap();
            let mapped_query_results: Vec<Vec<_>> = query_results.into_iter().map(|query_result| {
                query_result.as_slice().iter().map(Country::from_indexed_query_result_row).collect()
            }).collect();

            let mut expected_stmt = expected_conn.prepare(&sql).unwrap();
            let expected_results = queued_params.iter().map(|params| {
                let mapped_rows = expected_stmt.query_map(params, Country::from_indexed_rusqlite_row).unwrap();
                mapped_rows.map(|row| row.unwrap()).collect::<Vec<_>>()
            }).collect::<Vec<_>>();

            assert_eq!(expected_results, mapped_query_results);
        });
    }

    let no_param =
        include_str!("../res/sql/test_query_no_param.sql");
    let indexed_param =
        include_str!("../res/sql/test_query_indexed_param.sql");
    let indexed_params =
        include_str!("../res/sql/test_query_indexed_params.sql");

    for (sql, queued_params) in indexed_test_cases(no_param, indexed_param, indexed_params) {
        let queued_params_slices: Vec<_> = queued_params.iter().map(|vec| vec.as_slice()).collect();

        test_query_indexed_parameters(sql, AssertUnwindSafe(&queued_params_slices));
    }
}

#[test]
fn test_query_named() {
    fn test_query_named_parameters(sql: &str, queued_params: AssertUnwindSafe<&[&[(&str, &ToSql)]]>) {
        with_test_db_connections(ReadOnly, |mut test_conn: AccessConnection<ReadOnly>, expected_conn: Connection| {
            let queued_params = queued_params.0;

            let query = Query::new_named(&sql, queued_params).unwrap();
            let query_results = test_conn.run(&query).unwrap();
            let mapped_query_results: Vec<Vec<_>> = query_results.into_iter().map(|query_result| {
                query_result.as_slice().iter().map(Country::from_indexed_query_result_row).collect()
            }).collect();

            let mut expected_stmt = expected_conn.prepare(&sql).unwrap();
            let expected_results = queued_params.iter().map(|params| {
                let mapped_rows = expected_stmt.query_map_named(params, Country::from_indexed_rusqlite_row).unwrap();
                mapped_rows.map(|row| row.unwrap()).collect::<Vec<_>>()
            }).collect::<Vec<_>>();

            assert_eq!(expected_results, mapped_query_results);
        });
    }

    let no_param =
        include_str!("../res/sql/test_query_no_param.sql");
    let named_param =
        include_str!("../res/sql/test_query_named_param.sql");
    let named_params =
        include_str!("../res/sql/test_query_named_params.sql");

    for (sql, queued_params) in named_test_cases(no_param, named_param, named_params) {
        let queued_params_slices: Vec<_> = queued_params.iter().map(|vec| vec.as_slice()).collect();

        test_query_named_parameters(sql, AssertUnwindSafe(&queued_params_slices));
    }
}

#[test]
fn test_bulk_query() {
    with_test_db_connections(ReadOnly, |mut test_conn: AccessConnection<ReadOnly>, expected_conn: Connection| {
        let no_param =
            include_str!("../res/sql/test_query_no_param.sql");
        let indexed_param =
            include_str!("../res/sql/test_query_indexed_param.sql");
        let indexed_params =
            include_str!("../res/sql/test_query_indexed_params.sql");


        // Test
        let test_queries = indexed_test_cases(no_param, indexed_param, indexed_params).iter().map(|(sql, queued_params)| {
            let queued_params_slices: Vec<_> = queued_params.iter().map(|vec| vec.as_slice()).collect();

            Query::new_indexed(&sql, &queued_params_slices).unwrap()
        }).collect();

        let bulk_query = BulkQuery::new(test_queries);

        let bulk_query_results = test_conn.run(&bulk_query).unwrap();
        let mapped_bulk_query_results: Vec<_> = bulk_query_results.into_iter().map(|query_results| {
            let mapped_query_results: Vec<Vec<_>> = query_results.into_iter().map(|query_result| {
                query_result.as_slice().iter().map(Country::from_indexed_query_result_row).collect()
            }).collect();
            mapped_query_results
        }).collect();


        //Expected
        let bulk_expected_results: Vec<_> = indexed_test_cases(no_param, indexed_param, indexed_params).iter().map(|(sql, queued_params)| {
            let mut expected_stmt = expected_conn.prepare(&sql).unwrap();
            let expected_results = queued_params.iter().map(|params| {
                let mapped_rows = expected_stmt.query_map(params, Country::from_indexed_rusqlite_row).unwrap();
                mapped_rows.map(|row| row.unwrap()).collect::<Vec<_>>()
            }).collect::<Vec<_>>();
            expected_results
        }).collect();

        assert_eq!(mapped_bulk_query_results, bulk_expected_results);
    });
}

// Negative tests:


// TODO: test_query_err

// TODO: test_bulk_query_err (middle of transaction)
