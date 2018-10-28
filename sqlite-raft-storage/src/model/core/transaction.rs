use model::core::CoreId;
use rusqlite::Transaction;

pub struct CoreTx<'tx, 'conn: 'tx> {
    core_id: CoreId,
    tx: &'tx Transaction<'conn>,
}

impl<'tx, 'conn: 'tx> CoreTx<'tx, 'conn> {
    pub fn new(tx: &'tx Transaction<'conn>, core_id: CoreId) -> CoreTx<'tx, 'conn> {
        CoreTx {
            core_id,
            tx,
        }
    }

    pub fn tx(&self) -> &Transaction {
        &self.tx
    }

    pub fn core_id(&self) -> CoreId {
        self.core_id
    }
}

