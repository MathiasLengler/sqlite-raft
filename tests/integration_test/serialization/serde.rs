use bincode;
use integration_test::serialization::sqlite_requests;
use serde_json;
use sqlite_commands::request::SqliteRequest;
use sqlite_commands::request::SqliteResponse;
use integration_test::serialization::sqlite_responses;


#[test]
fn test_serde_requests() {
    let requests = sqlite_requests();

    let bincode_serialized = bincode::serialize(&requests).unwrap();
    let bincode_deserialized: Vec<SqliteRequest> = bincode::deserialize(&bincode_serialized).unwrap();

    let json_serialized = serde_json::to_string(&requests).unwrap();
    let json_deserialized: Vec<SqliteRequest> = serde_json::from_str(&json_serialized).unwrap();

    assert_eq!(requests, bincode_deserialized);
    assert_eq!(requests, json_deserialized);
}

#[test]
fn test_serde_responses() {
    let command_responses = sqlite_responses();

    let bincode_serialized = bincode::serialize(&command_responses).unwrap();
    let bincode_deserialized: Vec<SqliteResponse> = bincode::deserialize(&bincode_serialized).unwrap();

    let json_serialized = serde_json::to_string(&command_responses).unwrap();
    let json_deserialized: Vec<SqliteResponse> = serde_json::from_str(&json_serialized).unwrap();

    assert_eq!(command_responses, bincode_deserialized);
    assert_eq!(command_responses, json_deserialized);
}