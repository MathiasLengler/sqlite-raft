use error::Result;
use rusqlite::Transaction;
use rusqlite::types::ToSql;
pub use self::transaction::CoreTx;

mod transaction;

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Ord, Eq)]
pub struct CoreId(i64);

impl CoreId {
    const SQL_EXISTS: &'static str = include_str!("../../../res/sql/core/exists.sql");
    const SQL_INSERT: &'static str = include_str!("../../../res/sql/core/insert.sql");

    pub fn as_named_param(&self) -> (&'static str, &ToSql) {
        (":core_id", &self.0)
    }

    pub fn exists(&self, tx: &Transaction) -> Result<bool> {
        let mut exists_stmt = tx.prepare(CoreId::SQL_EXISTS)?;

        Ok(exists_stmt.exists(&[&self.0])?)
    }

    pub fn insert(&self, tx: &Transaction) -> Result<()> {
        tx.execute_named(
            CoreId::SQL_INSERT,
            &[self.as_named_param()],
        )?;

        Ok(())
    }
}

impl From<i64> for CoreId {
    fn from(id: i64) -> Self {
        CoreId(id)
    }
}
