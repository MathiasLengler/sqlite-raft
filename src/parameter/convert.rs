use parameter::IndexedParameters;
use parameter::NamedParameter;
use parameter::NamedParameters;
use parameter::QueuedParameters;
use proto::ProtoIndexedParameters;
use proto::ProtoNamedParameter;
use proto::ProtoNamedParameters;
use proto::ProtoNull;
use proto::ProtoQueuedIndexedParameters;
use proto::ProtoQueuedNamedParameters;
use proto::ProtoQueuedParameters;
use proto::ProtoQueuedParameters_oneof_queued_parameters;
use proto::ProtoValue;
use proto::ProtoValue_oneof_value;
use rusqlite::types::Value;

impl From<QueuedParameters> for ProtoQueuedParameters {
    fn from(queued_parameters: QueuedParameters) -> Self {
        let mut proto_query_request = ProtoQueuedParameters::new();
        match queued_parameters {
            QueuedParameters::Indexed(indexed) =>
                proto_query_request.set_queued_indexed_parameters(indexed.into()),
            QueuedParameters::Named(named) =>
                proto_query_request.set_queued_named_parameters(named.into()),
        }
        proto_query_request
    }
}

impl From<ProtoQueuedParameters> for QueuedParameters {
    fn from(proto_queued_parameters: ProtoQueuedParameters) -> Self {
        match proto_queued_parameters.queued_parameters.unwrap() {
            ProtoQueuedParameters_oneof_queued_parameters::queued_indexed_parameters(
                proto_queued_indexed_parameters
            ) => {
                QueuedParameters::Indexed(proto_queued_indexed_parameters.into())
            }
            ProtoQueuedParameters_oneof_queued_parameters::queued_named_parameters(
                proto_queued_named_parameters
            ) => {
                QueuedParameters::Named(proto_queued_named_parameters.into())
            }
        }
    }
}


impl From<Vec<IndexedParameters>> for ProtoQueuedIndexedParameters {
    fn from(vec_indexed_parameters: Vec<IndexedParameters>) -> Self {
        let mut proto_queued_indexed_parameters = ProtoQueuedIndexedParameters::new();
        let vec_proto_indexed_parameters: Vec<ProtoIndexedParameters> =
            vec_indexed_parameters.into_iter().map(Into::into).collect();
        proto_queued_indexed_parameters.set_queued_indexed_parameters(vec_proto_indexed_parameters.into());
        proto_queued_indexed_parameters
    }
}

impl From<ProtoQueuedIndexedParameters> for Vec<IndexedParameters> {
    fn from(mut proto_queued_indexed_parameters: ProtoQueuedIndexedParameters) -> Self {
        proto_queued_indexed_parameters
            .take_queued_indexed_parameters()
            .into_vec()
            .into_iter()
            .map(Into::into)
            .collect()
    }
}

impl From<Vec<NamedParameters>> for ProtoQueuedNamedParameters {
    fn from(vec_named_parameters: Vec<NamedParameters>) -> Self {
        let mut proto_queued_named_parameters = ProtoQueuedNamedParameters::new();
        let vec_proto_named_parameters: Vec<ProtoNamedParameters> =
            vec_named_parameters.into_iter().map(Into::into).collect();
        proto_queued_named_parameters.set_queued_named_parameters(vec_proto_named_parameters.into());
        proto_queued_named_parameters
    }
}

impl From<ProtoQueuedNamedParameters> for Vec<NamedParameters> {
    fn from(mut proto_queued_named_parameters: ProtoQueuedNamedParameters) -> Self {
        proto_queued_named_parameters
            .take_queued_named_parameters()
            .into_vec()
            .into_iter()
            .map(Into::into)
            .collect()
    }
}

impl From<IndexedParameters> for ProtoIndexedParameters {
    fn from(indexed_parameters: IndexedParameters) -> Self {
        let mut proto_indexed_parameters = ProtoIndexedParameters::new();
        let vec_proto_value: Vec<ProtoValue> =
            indexed_parameters.parameters.into_iter().map(Into::into).collect();
        proto_indexed_parameters.set_parameters(vec_proto_value.into());
        proto_indexed_parameters
    }
}

impl From<ProtoIndexedParameters> for IndexedParameters {
    fn from(mut proto_indexed_parameters: ProtoIndexedParameters) -> Self {
        IndexedParameters {
            parameters: proto_indexed_parameters
                .take_parameters()
                .into_vec()
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

impl From<NamedParameters> for ProtoNamedParameters {
    fn from(named_parameters: NamedParameters) -> Self {
        let mut proto_named_parameters = ProtoNamedParameters::new();
        let vec_proto_named_parameter: Vec<ProtoNamedParameter> =
            named_parameters.parameters.into_iter().map(Into::into).collect();
        proto_named_parameters.set_parameters(vec_proto_named_parameter.into());
        proto_named_parameters
    }
}

impl From<ProtoNamedParameters> for NamedParameters {
    fn from(mut proto_named_parameters: ProtoNamedParameters) -> Self {
        NamedParameters {
            parameters: proto_named_parameters
                .take_parameters()
                .into_vec()
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

impl From<NamedParameter> for ProtoNamedParameter {
    fn from(named_parameter: NamedParameter) -> Self {
        let mut proto_named_parameter = ProtoNamedParameter::new();
        proto_named_parameter.set_name(named_parameter.name);
        proto_named_parameter.set_value(named_parameter.value.into());
        proto_named_parameter
    }
}

impl From<ProtoNamedParameter> for NamedParameter {
    fn from(mut proto_named_parameter: ProtoNamedParameter) -> Self {
        NamedParameter {
            name: proto_named_parameter.take_name(),
            value: proto_named_parameter.take_value().into(),
        }
    }
}

// TODO: move to value module
impl From<Value> for ProtoValue {
    fn from(value: Value) -> Self {
        let mut proto_value = ProtoValue::new();

        match value {
            Value::Null => proto_value.set_null(ProtoNull::new()),
            Value::Integer(i) => proto_value.set_integer(i),
            Value::Real(f) => proto_value.set_real(f),
            Value::Text(text) => proto_value.set_text(text),
            Value::Blob(blob) => proto_value.set_blob(blob),
        };

        proto_value
    }
}

impl From<ProtoValue> for Value {
    fn from(proto_value: ProtoValue) -> Self {
        match proto_value.value.unwrap() {
            ProtoValue_oneof_value::null(_) => Value::Null,
            ProtoValue_oneof_value::integer(i) => Value::Integer(i),
            ProtoValue_oneof_value::real(f) => Value::Real(f),
            ProtoValue_oneof_value::text(text) => Value::Text(text),
            ProtoValue_oneof_value::blob(blob) => Value::Blob(blob),
        }
    }
}
