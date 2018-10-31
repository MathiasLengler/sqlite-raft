use crate::integration_test::serialization::sqlite_requests;
use crate::integration_test::serialization::sqlite_responses;
use sqlite_requests::proto::ProtoSqliteRequest;
use sqlite_requests::proto::ProtoSqliteResponse;
use sqlite_requests::request::SqliteRequest;
use sqlite_requests::request::SqliteResponse;

#[test]
fn test_proto_requests() {
    let sqlite_requests = sqlite_requests();

    let converted_sqlite_requests: Vec<SqliteRequest> = sqlite_requests.clone().into_iter()
        .map(Into::<ProtoSqliteRequest>::into)
        .map(Into::into)
        .collect();

    assert_eq!(sqlite_requests, converted_sqlite_requests);
}

#[test]
fn test_proto_responses() {
    let sqlite_responses = sqlite_responses();

    let converted_sqlite_responses: Vec<SqliteResponse> = sqlite_responses.clone().into_iter()
        .map(Into::<ProtoSqliteResponse>::into)
        .map(Into::into)
        .collect();

    assert_eq!(sqlite_responses, converted_sqlite_responses);
}
