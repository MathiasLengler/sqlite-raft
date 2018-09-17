use connection::AccessTransaction;
use connection::Request;
use connection::ReadOnly;
use error::Result;
use parameter::IndexedParameters;
use parameter::NamedParameters;
use parameter::QueuedParameters;
use rusqlite;
use rusqlite::Row;
use rusqlite::Statement;
use rusqlite::types::FromSql;
use rusqlite::types::ToSql;
use rusqlite::types::Value;
use rusqlite::types::ValueRef;
use std::result;

mod proto_convert;

/// Execute a series of queries. Each query can be run with multiple different parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BulkQuery {
    queries: Vec<Query>,
}

impl BulkQuery {
    pub fn new(queries: Vec<Query>) -> BulkQuery {
        BulkQuery {
            queries,
        }
    }
}

impl Request for BulkQuery {
    type Access = ReadOnly;
    type Response = Vec<Vec<QueryResponse>>;

    fn apply_to_tx(&self, tx: &mut AccessTransaction<Self::Access>) -> Result<Self::Response> {
        self.queries.iter().map(|query| {
            query.apply_to_tx(tx)
        }).collect::<Result<Vec<_>>>()
    }
}

/// Execute a single query once or multiple times with different parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Query {
    sql: String,
    queued_parameters: QueuedParameters,
}

impl Query {
    // TODO: add single non-queued parameter convenience constructor (return value?)

    pub fn new_indexed(sql: &str, queued_indexed_parameters: &[&[&ToSql]]) -> Result<Query> {
        Ok(Query {
            sql: sql.to_string(),
            queued_parameters: QueuedParameters::new_indexed(queued_indexed_parameters)?,
        })
    }

    pub fn new_named(sql: &str, queued_named_parameters: &[&[(&str, &ToSql)]]) -> Result<Query> {
        Ok(Query {
            sql: sql.to_string(),
            queued_parameters: QueuedParameters::new_named(queued_named_parameters)?,
        })
    }
}

impl Request for Query {
    type Access = ReadOnly;
    type Response = Vec<QueryResponse>;

    fn apply_to_tx(&self, tx: &mut AccessTransaction<Self::Access>) -> Result<Self::Response> {
        let tx = tx.as_mut_inner();
        let mut stmt = tx.prepare(&self.sql)?;

        let res = self.queued_parameters.map_parameter_variants(
            &mut stmt,
            |stmt: &mut Statement, parameters: &IndexedParameters| {
                let rows = stmt.query_map(
                    &parameters.as_arg(),
                    QueryResultRow::query_map_arg(),
                )?;

                QueryResponse::try_from(rows)
            },
            |stmt: &mut Statement, parameters: &NamedParameters| {
                let rows = stmt.query_map_named(
                    &parameters.as_arg(),
                    QueryResultRow::query_map_arg(),
                )?;

                QueryResponse::try_from(rows)
            },
        );

        res
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryResponse {
    rows: Vec<QueryResultRow>,
}

impl QueryResponse {
    pub fn into_vec(self) -> Vec<QueryResultRow> {
        self.rows
    }

    pub fn as_slice(&self) -> &[QueryResultRow] {
        &self.rows
    }

    fn try_from(rows_iter: impl Iterator<Item=result::Result<QueryResultRow, rusqlite::Error>>)
                -> Result<QueryResponse> {
        let rows: result::Result<Vec<QueryResultRow>, rusqlite::Error> = rows_iter.collect();

        Ok(QueryResponse {
            rows: rows?,
        })
    }
}

// TODO: rename QueryResponseRow
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryResultRow {
    #[serde(with = "::value::serde")]
    row: Vec<Value>
}

impl QueryResultRow {
    // TODO: named column index (via Statement::column_names field)
    // TODO: get checked

    pub fn get<T: FromSql>(&self, idx: usize) -> T {
        let value = &self.row[idx];
        let value_ref: ValueRef = From::from(value);

        FromSql::column_result(value_ref).unwrap()
    }

    pub fn into_vec(self) -> Vec<Value> {
        self.row
    }

    pub fn as_slice(&self) -> &[Value] {
        &self.row
    }

    fn query_map_arg() -> impl FnMut(&Row) -> QueryResultRow {
        |row: &Row| {
            let row: Vec<_> = (0..row.column_count())
                .map(|row_index| {
                    let value: Value = row.get(row_index);
                    value
                }).collect();
            QueryResultRow {
                row,
            }
        }
    }
}
