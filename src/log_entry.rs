use bincode::{self, Result as BincodeResult};

#[derive(Debug)]
pub struct LogEntryFactory {
    node_id: u64,
    next_propose_id: u64,
}

impl LogEntryFactory {
    pub fn new(node_id: u64, next_propose_id: u64) -> LogEntryFactory {
        LogEntryFactory {
            node_id,
            next_propose_id,
        }
    }

    pub fn new_log_entry(&mut self, text: String) -> LogEntry {
        let propose_id = self.next_propose_id;

        self.next_propose_id = self.next_propose_id.wrapping_add(1);

        // TODO: persist next_propose_id

        LogEntry {
            key: LogEntryKey {
                node_id: self.node_id,
                propose_id,
            },
            text,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    key: LogEntryKey,
    /// The text stored in the raft log, e.g. SQL-Statement.
    text: String,
}

impl LogEntry {
    pub fn try_from(data: &[u8]) -> BincodeResult<LogEntry> {
        bincode::deserialize(&data)
    }

    pub fn to_vec_u8(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }

    pub fn key(&self) -> LogEntryKey {
        self.key
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct LogEntryKey {
    /// From which node the log entry has been added.
    node_id: u64,
    /// Unique id per node. Used to differentiate entries with equal data and node_id.
    propose_id: u64,
}