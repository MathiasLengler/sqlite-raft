use rusqlite::types::Value;
use serde::{Deserialize, Deserializer, Serializer};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(remote = "Value")]
pub enum ValueDef {
    /// The value is a `NULL` value.
    Null,
    /// The value is a signed integer.
    Integer(i64),
    /// The value is a floating point number.
    Real(f64),
    /// The value is a text string.
    Text(String),
    /// The value is a blob of data
    Blob(Vec<u8>),
}

pub fn serialize<S>(array: &[Value], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
{
    #[derive(Serialize)]
    struct W<'a>(#[serde(with = "ValueDef")] &'a Value);

    let values = array.iter().map(|value| W(value));
    serializer.collect_seq(values)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Value>, D::Error>
    where
        D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct W(#[serde(with = "ValueDef")] Value);

    let values = Vec::<W>::deserialize(deserializer)?;
    Ok(values.into_iter().map(|w_value| w_value.0).collect())
}
