use sqlite_commands::QueryResultRow;
use rusqlite::Row;

#[derive(Debug, Eq, PartialEq)]
pub struct Country {
    rank: i32,
    name: String,
    alpha_2: String,
    alpha_3: String,
}

impl Country {
    pub fn from_indexed_query_result_row(row: &QueryResultRow) -> Self {
        let rank: i32 = row.get(0);
        let name: String = row.get(1);
        let alpha_2: String = row.get(2);
        let alpha_3: String = row.get(3);
        Country { rank, name, alpha_2, alpha_3 }
    }

    pub fn from_indexed_rusqlite_row(row: &Row) -> Self {
        let rank: i32 = row.get(0);
        let name: String = row.get(1);
        let alpha_2: String = row.get(2);
        let alpha_3: String = row.get(3);
        Country { rank, name, alpha_2, alpha_3 }
    }
}

