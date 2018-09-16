use sqlite_requests::query::Query;
use sqlite_requests::execute::Execute;
use sqlite_requests::query::BulkQuery;
use sqlite_requests::execute::BulkExecute;
use integration_test::indexed_test_cases;
use integration_test::named_test_cases;
use integration_test::queued_params_as_arg;
use sqlite_requests::request::SqliteRequest;
use sqlite_requests::request::SqliteResponse;
use sqlite_requests::connection::AccessConnection;
use sqlite_requests::connection::ReadOnly;
use sqlite_requests::connection::ReadWrite;
use utils::temp_db::with_single_test_db;

mod serde;
mod proto;

struct SerializationRequests {
    queries: Vec<Query>,
    executes: Vec<Execute>,
    bulk_query: BulkQuery,
    bulk_execute: BulkExecute,
}

fn serialization_requests() -> SerializationRequests {
    let query_indexed_test_cases = indexed_test_cases(
        include_str!("../../res/sql/test_query_no_param.sql"),
        include_str!("../../res/sql/test_query_indexed_param.sql"),
        include_str!("../../res/sql/test_query_indexed_params.sql"),
    );
    let query_named_test_cases = named_test_cases(
        include_str!("../../res/sql/test_query_no_param.sql"),
        include_str!("../../res/sql/test_query_named_param.sql"),
        include_str!("../../res/sql/test_query_named_params.sql"),
    );
    let execute_indexed_test_cases = indexed_test_cases(
        include_str!("../../res/sql/test_execute_no_param.sql"),
        include_str!("../../res/sql/test_execute_indexed_param.sql"),
        include_str!("../../res/sql/test_execute_indexed_params.sql"),
    );
    let execute_named_test_cases = named_test_cases(
        include_str!("../../res/sql/test_execute_no_param.sql"),
        include_str!("../../res/sql/test_execute_named_param.sql"),
        include_str!("../../res/sql/test_execute_named_params.sql"),
    );

    let queries: Vec<Query> = query_indexed_test_cases.iter()
        .map(|(sql, queued_params)| {
            let queued_params = queued_params_as_arg(queued_params);
            Query::new_indexed(sql, &queued_params).unwrap()
        })
        .chain(
            query_named_test_cases.iter().map(|(sql, queued_params)| {
                let queued_params = queued_params_as_arg(queued_params);
                Query::new_named(sql, &queued_params).unwrap()
            })
        ).collect();

    let executes: Vec<Execute> = execute_indexed_test_cases.iter()
        .map(|(sql, queued_params)| {
            let queued_params = queued_params_as_arg(queued_params);
            Execute::new_indexed(sql, &queued_params).unwrap()
        })
        .chain(
            execute_named_test_cases.iter().map(|(sql, queued_params)| {
                let queued_params = queued_params_as_arg(queued_params);
                Execute::new_named(sql, &queued_params).unwrap()
            })
        )
        .collect();

    let bulk_query = BulkQuery::new(queries.clone());
    let bulk_execute = BulkExecute::new(executes.clone());

    SerializationRequests {
        queries,
        executes,
        bulk_query,
        bulk_execute,
    }
}

fn sqlite_requests() -> Vec<SqliteRequest> {
    let SerializationRequests {
        queries,
        executes,
        bulk_query,
        bulk_execute,
    } = serialization_requests();

    let mut commands: Vec<SqliteRequest> = vec![bulk_execute.into(), bulk_query.into(), ];

    commands.extend(queries.into_iter().map(Into::into));
    commands.extend(executes.into_iter().map(Into::into));

    commands
}

fn sqlite_responses() -> Vec<SqliteResponse> {
    let commands = sqlite_requests();

    commands.iter().map(|command| {
        with_single_test_db(|mut conn_ro: AccessConnection<ReadOnly>, mut conn_rw: AccessConnection<ReadWrite>| {
            match command {
                SqliteRequest::Query(sqlite_query) => {
                    SqliteResponse::Query(conn_ro.run(sqlite_query).unwrap())
                }
                SqliteRequest::Execute(sqlite_execute) => {
                    SqliteResponse::Execute(conn_rw.run(sqlite_execute).unwrap())
                }
            }
        })
    }).collect()
}
