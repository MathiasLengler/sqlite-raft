use crate::proto::ProtoNull;
use crate::proto::ProtoValue;
use crate::proto::ProtoValue_oneof_value;
use rusqlite::types::Value;

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
