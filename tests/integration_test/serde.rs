use bincode;
use integration_test::indexed_test_cases;
use integration_test::named_test_cases;
use integration_test::queued_params_as_arg;
use serde_json;
use sqlite_commands::execute::BulkExecute;
use sqlite_commands::execute::Execute;
use sqlite_commands::query::BulkQuery;
use sqlite_commands::query::Query;
use sqlite_commands::SqliteCommand;


#[test]
fn test_serde() {
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

    let executes: Vec<Execute> = indexed_test_cases.iter()
        .map(|(sql, queued_params)| {
            let queued_params = queued_params_as_arg(queued_params);
            Execute::new_indexed(sql, &queued_params).unwrap()
        })
        .chain(
            named_test_cases.iter().map(|(sql, queued_params)| {
                let queued_params = queued_params_as_arg(queued_params);
                Execute::new_named(sql, &queued_params).unwrap()
            })
        )
        .collect();

    let bulk_query = BulkQuery::new(queries.clone());
    let bulk_execute = BulkExecute::new(executes.clone());

    let mut commands: Vec<SqliteCommand> = vec![bulk_execute.into(), bulk_query.into(), ];

    commands.extend(queries.into_iter().map(Into::into));
    commands.extend(executes.into_iter().map(Into::into));

    let bincode_serialized = bincode::serialize(&commands).unwrap();
    let bincode_deserialized: Vec<SqliteCommand> = bincode::deserialize(&bincode_serialized).unwrap();

    let json_serialized = serde_json::to_string(&commands).unwrap();
    let json_deserialized: Vec<SqliteCommand> = serde_json::from_str(&json_serialized).unwrap();

    assert_eq!(commands, bincode_deserialized);
    assert_eq!(commands, json_deserialized);

    // TODO: execute and test result sets
}
