use integration_test::indexed_test_cases;
use integration_test::named_test_cases;
use integration_test::queued_params_as_arg;
use sqlite_commands::execute::Execute;
use sqlite_commands::proto::ProtoExecuteRequest;
use sqlite_commands::proto::ProtoQueryRequest;
use sqlite_commands::query::Query;

// TODO: refactor using sqlite_commands()

#[test]
fn test_proto_query() {
    let indexed_test_cases = indexed_test_cases("foo", "bar", "baz");
    let named_test_cases = named_test_cases("foo", "bar", "baz");

    let queries: Vec<Query> = indexed_test_cases.iter()
        .map(|(sql, queued_params)| {
            let queued_params = queued_params_as_arg(queued_params);
            Query::new_indexed(sql, &queued_params).unwrap()
        })
        .chain(
            named_test_cases.iter().map(|(sql, queued_params)| {
                let queued_params = queued_params_as_arg(queued_params);
                Query::new_named(sql, &queued_params).unwrap()
            })
        ).collect();

    let converted_queries: Vec<Query> = queries.clone().into_iter()
        .map(Into::<ProtoQueryRequest>::into)
        .map(Into::into)
        .collect();

    assert_eq!(queries, converted_queries);
}

#[test]
fn test_proto_execute() {
    let indexed_test_cases = indexed_test_cases("foo", "bar", "baz");
    let named_test_cases = named_test_cases("foo", "bar", "baz");

    let queries: Vec<Execute> = indexed_test_cases.iter()
        .map(|(sql, queued_params)| {
            let queued_params = queued_params_as_arg(queued_params);
            Execute::new_indexed(sql, &queued_params).unwrap()
        })
        .chain(
            named_test_cases.iter().map(|(sql, queued_params)| {
                let queued_params = queued_params_as_arg(queued_params);
                Execute::new_named(sql, &queued_params).unwrap()
            })
        ).collect();

    let converted_queries: Vec<Execute> = queries.clone().into_iter()
        .map(Into::<ProtoExecuteRequest>::into)
        .map(Into::into)
        .collect();

    assert_eq!(queries, converted_queries);
}
