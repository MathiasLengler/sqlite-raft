use error::Result;
use rusqlite::types::ToSql;
use rusqlite::types::Value;
use rusqlite::Statement;
use rusqlite::types::ValueRef;
use rusqlite::types::ToSqlOutput;

pub enum QueuedParameters {
    Indexed(Vec<IndexedParameters>),
    Named(Vec<NamedParameters>),
}

impl QueuedParameters {
    fn new() -> QueuedParameters {
        // TODO: ensure at least one parameter set in each variant.
        unimplemented!()
    }

    pub(crate) fn map_parameter_variants<T>(&self,
                                            stmt: &mut Statement,
                                            mut indexed: impl FnMut(&mut Statement, &IndexedParameters) -> Result<T>,
                                            mut named: impl FnMut(&mut Statement, &NamedParameters) -> Result<T>)
                                            -> Result<Vec<T>> {
        match self {
            QueuedParameters::Indexed(ref queued_indexed_parameters) => {
                queued_indexed_parameters.iter().map(|parameters| {
                    indexed(stmt, parameters)
                }).collect()
            }
            QueuedParameters::Named(ref queued_named_parameters) => {
                queued_named_parameters.iter().map(|parameters| {
                    named(stmt, parameters)
                }).collect()
            }
        }
    }
}

pub struct IndexedParameters {
    parameters: Vec<Value>
}

impl IndexedParameters {
    fn new<T>(parameters: &[&ToSql]) -> IndexedParameters {
        use parameter::ToValue;

        IndexedParameters {
            parameters: parameters.iter().map(|parameter| {
                let value: ToSqlOutput = parameter.to_sql().unwrap().to_value();
            }).collect(),
        }
    }

    pub(crate) fn as_arg(&self) -> Vec<&ToSql> {
        self.parameters.iter().map(|value| value as &ToSql).collect()
    }
}

trait ToValue {
    fn into_value(self) -> Value;
}

impl<'a> ToValue for ToSqlOutput<'a> {
    fn into_value(self) -> Value {
        match self {
            ToSqlOutput::Borrowed(value_ref) => {value_ref.into()},
            ToSqlOutput::Owned(value) => {value},
            #[cfg(feature = "blob")]
            ToSqlOutput::ZeroBlob(length) => { unimplemented!()},
        }
    }
}

pub struct NamedParameters {
    parameters: Vec<NamedParameter>
}

impl NamedParameters {
    pub(crate) fn as_arg(&self) -> Vec<(&str, &ToSql)> {
        self.parameters.iter().map(
            |NamedParameter {
                 name,
                 value,
             }| {
                (name.as_str(), value as &ToSql)
            }).collect()
    }
}

pub struct NamedParameter {
    name: String,
    value: Value,
}