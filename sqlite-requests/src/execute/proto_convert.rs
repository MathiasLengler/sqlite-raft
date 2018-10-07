use execute::BulkExecute;
use execute::Execute;
use execute::ExecuteResult;
use proto::ProtoExecuteRequest;
use proto::ProtoExecuteResponse;
use proto::ProtoExecuteResult;
use proto::ProtoBulkExecuteRequest;
use proto::ProtoBulkExecuteResponse;

impl From<Execute> for ProtoExecuteRequest {
    fn from(execute: Execute) -> Self {
        let mut proto_execute_request = ProtoExecuteRequest::new();
        proto_execute_request.set_sql(execute.sql);
        proto_execute_request.set_queued_parameters(execute.queued_parameters.into());
        proto_execute_request
    }
}

impl From<ProtoExecuteRequest> for Execute {
    fn from(mut proto_execute_request: ProtoExecuteRequest) -> Self {
        Execute {
            sql: proto_execute_request.take_sql(),
            queued_parameters: proto_execute_request.take_queued_parameters().into(),
        }
    }
}

impl From<Vec<ExecuteResult>> for ProtoExecuteResponse {
    fn from(vec_execute_result: Vec<ExecuteResult>) -> Self {
        let mut proto_execute_response = ProtoExecuteResponse::new();
        let vec_proto_execute_result: Vec<ProtoExecuteResult> =
            vec_execute_result.into_iter().map(Into::into).collect();
        proto_execute_response.set_execute_results(vec_proto_execute_result.into());
        proto_execute_response
    }
}

impl From<ProtoExecuteResponse> for Vec<ExecuteResult> {
    fn from(mut proto_execute_response: ProtoExecuteResponse) -> Self {
        proto_execute_response
            .take_execute_results()
            .into_vec()
            .into_iter()
            .map(Into::into)
            .collect()
    }
}

impl From<ExecuteResult> for ProtoExecuteResult {
    fn from(execute_result: ExecuteResult) -> Self {
        let mut proto_execute_result = ProtoExecuteResult::new();
        proto_execute_result.set_changes(execute_result.changes as u64);
        proto_execute_result
    }
}

impl From<ProtoExecuteResult> for ExecuteResult {
    fn from(proto_execute_result: ProtoExecuteResult) -> Self {
        ExecuteResult {
            changes: proto_execute_result.get_changes() as usize,
        }
    }
}


impl From<BulkExecute> for ProtoBulkExecuteRequest {
    fn from(bulk_execute: BulkExecute) -> Self {
        let mut proto_bulk_execute_request = ProtoBulkExecuteRequest::new();
        let vec_proto_execute_request: Vec<ProtoExecuteRequest> =
            bulk_execute.executes.into_iter().map(Into::into).collect();
        proto_bulk_execute_request.set_executes(vec_proto_execute_request.into());
        proto_bulk_execute_request
    }
}

impl From<ProtoBulkExecuteRequest> for BulkExecute {
    fn from(mut proto_bulk_execute_request: ProtoBulkExecuteRequest) -> Self {
        BulkExecute {
            executes: proto_bulk_execute_request
                .take_executes()
                .into_vec()
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

impl From<Vec<Vec<ExecuteResult>>> for ProtoBulkExecuteResponse {
    fn from(vec_vec_execute_response: Vec<Vec<ExecuteResult>>) -> Self {
        let mut proto_bulk_execute_response = ProtoBulkExecuteResponse::new();
        let vec_proto_execute_response: Vec<ProtoExecuteResponse> =
            vec_vec_execute_response.into_iter().map(Into::into).collect();
        proto_bulk_execute_response.set_execute_responses(vec_proto_execute_response.into());
        proto_bulk_execute_response
    }
}

impl From<ProtoBulkExecuteResponse> for Vec<Vec<ExecuteResult>> {
    fn from(mut proto_bulk_execute_response: ProtoBulkExecuteResponse) -> Self {
        proto_bulk_execute_response
            .take_execute_responses()
            .into_vec()
            .into_iter()
            .map(Into::into)
            .collect()
    }
}
