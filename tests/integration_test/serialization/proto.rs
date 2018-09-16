use integration_test::indexed_test_cases;
use integration_test::named_test_cases;
use integration_test::queued_params_as_arg;
use integration_test::serialization::sqlite_responses;
use integration_test::serialization::sqlite_requests;
use sqlite_requests::execute::Execute;
use sqlite_requests::proto::ProtoExecuteRequest;
use sqlite_requests::proto::ProtoQueryRequest;
use sqlite_requests::query::Query;
use sqlite_requests::request::SqliteRequest;
use sqlite_requests::request::SqliteExecute;
use sqlite_requests::request::SqliteQuery;

// TODO: refactor using sqlite_requests()

#[test]
fn test_proto_requests() {
    let requests = sqlite_requests();


}

#[test]
fn test_proto_responses() {
    let responses = sqlite_responses();
}

//#[test]
//fn test_proto_query() {
//    let converted_queries: Vec<Query> = queries.clone().into_iter()
//        .map(Into::<ProtoQueryRequest>::into)
//        .map(Into::into)
//        .collect();
//
//    assert_eq!(queries, converted_queries);
//}
//
//#[test]
//fn test_proto_execute() {
//
//
//    let converted_queries: Vec<Execute> = queries.clone().into_iter()
//        .map(Into::<ProtoExecuteRequest>::into)
//        .map(Into::into)
//        .collect();
//
//    assert_eq!(queries, converted_queries);
//}
