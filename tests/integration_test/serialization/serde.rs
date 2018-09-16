use bincode;
use integration_test::serialization::sqlite_commands;
use serde_json;
use sqlite_commands::connection::AccessConnection;
use sqlite_commands::connection::ReadOnly;
use sqlite_commands::connection::ReadWrite;
use sqlite_commands::SqliteCommand;
use sqlite_commands::SqliteCommandResult;
use utils::temp_db::with_single_test_db;


#[test]
fn test_serde_commands() {
    let commands = sqlite_commands();

    let bincode_serialized = bincode::serialize(&commands).unwrap();
    let bincode_deserialized: Vec<SqliteCommand> = bincode::deserialize(&bincode_serialized).unwrap();

    let json_serialized = serde_json::to_string(&commands).unwrap();
    let json_deserialized: Vec<SqliteCommand> = serde_json::from_str(&json_serialized).unwrap();

    assert_eq!(commands, bincode_deserialized);
    assert_eq!(commands, json_deserialized);
}

#[test]
fn test_serde_results() {
    let commands = sqlite_commands();

    let command_results: Vec<SqliteCommandResult> = commands.iter().map(|command| {
        with_single_test_db(|mut conn_ro: AccessConnection<ReadOnly>, mut conn_rw: AccessConnection<ReadWrite>| {
            match command {
                SqliteCommand::Query(sqlite_query) => {
                    SqliteCommandResult::Query(conn_ro.run(sqlite_query).unwrap())
                }
                SqliteCommand::Execute(sqlite_execute) => {
                    SqliteCommandResult::Execute(conn_rw.run(sqlite_execute).unwrap())
                }
            }
        })
    }).collect();

    let bincode_serialized = bincode::serialize(&command_results).unwrap();
    let bincode_deserialized: Vec<SqliteCommandResult> = bincode::deserialize(&bincode_serialized).unwrap();

    let json_serialized = serde_json::to_string(&command_results).unwrap();
    let json_deserialized: Vec<SqliteCommandResult> = serde_json::from_str(&json_serialized).unwrap();

    assert_eq!(command_results, bincode_deserialized);
    assert_eq!(command_results, json_deserialized);
}