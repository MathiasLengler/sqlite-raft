use error::Error;
use error::Result;
use rusqlite::Statement;
use rusqlite::types::ToSql;
use rusqlite::types::ToSqlOutput;
use rusqlite::types::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum QueuedParameters {
    Indexed(Vec<IndexedParameters>),
    Named(Vec<NamedParameters>),
}

impl QueuedParameters {
    fn assert_slice<T>(queued_parameters: &[T]) -> Result<()> {
        if queued_parameters.len() == 0 {
            return Err(Error::NoQueuedParameters);
        }

        Ok(())
    }

    pub(crate) fn new_indexed(queued_indexed_parameters: &[&[&ToSql]]) -> Result<QueuedParameters> {
        Self::assert_slice(queued_indexed_parameters)?;

        let queued_vec = queued_indexed_parameters.iter()
            .map(|parameter| IndexedParameters::new(parameter))
            .collect::<Result<Vec<_>>>()?;

        Ok(QueuedParameters::Indexed(queued_vec))
    }

    pub(crate) fn new_named(queued_named_parameters: &[&[(&str, &ToSql)]]) -> Result<QueuedParameters> {
        Self::assert_slice(queued_named_parameters)?;

        let queued_vec = queued_named_parameters.iter()
            .map(|parameter| NamedParameters::new(parameter))
            .collect::<Result<Vec<_>>>()?;

        Ok(QueuedParameters::Named(queued_vec))
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

/// Helper Trait for converting a `ToSql` into a `Value`.
trait IntoValue {
    fn into_value(self) -> Value;
}

impl<'a> IntoValue for ToSqlOutput<'a> {
    fn into_value(self) -> Value {
        match self {
            ToSqlOutput::Borrowed(value_ref) => { value_ref.into() }
            ToSqlOutput::Owned(value) => { value }
            #[cfg(feature = "blob")]
            ToSqlOutput::ZeroBlob(length) => { vec![0u8; length as usize].into() }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IndexedParameters {
    parameters: Vec<Value>
}

impl IndexedParameters {
    fn new(parameters: &[&ToSql]) -> Result<IndexedParameters> {
        Ok(IndexedParameters {
            parameters: parameters.iter().map(|parameter| {
                Ok(parameter.to_sql()?.into_value())
            }).collect::<Result<Vec<_>>>()?,
        })
    }

    pub(crate) fn as_arg(&self) -> Vec<&ToSql> {
        self.parameters.iter().map(|value| value as &ToSql).collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamedParameters {
    parameters: Vec<NamedParameter>
}

impl NamedParameters {
    fn new(parameters: &[(&str, &ToSql)]) -> Result<NamedParameters> {
        Ok(NamedParameters {
            parameters: parameters.iter().map(|(name, value)| {
                Ok(NamedParameter {
                    name: name.to_string(),
                    value: value.to_sql()?.into_value(),
                })
            }).collect::<Result<Vec<_>>>()?,
        })
    }

    pub(crate) fn as_arg(&self) -> Vec<(&str, &ToSql)> {
        self.parameters.iter().map(NamedParameter::as_arg).collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamedParameter {
    name: String,
    value: Value,
}

impl NamedParameter {
    fn as_arg(&self) -> (&str, &ToSql) {
        (self.name.as_str(), &self.value as &ToSql)
    }
}