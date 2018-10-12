use failure::Backtrace;
use raft::Error as RaftError;
use raft::StorageError as RaftStorageError;
use std::fmt;
use std::fmt::Display;

#[derive(Debug, Fail, Default)]
pub struct InvalidEntryIndex {
    pub kind: BoundViolation,
    pub first_index: u64,
    pub last_index: u64,
    pub invalid_index: u64,
    pub backtrace: Backtrace,
}

impl From<BoundViolation> for InvalidEntryIndex {
    fn from(kind: BoundViolation) -> Self {
        InvalidEntryIndex {
            kind,
            ..Default::default()
        }
    }
}


impl From<InvalidEntryIndex> for RaftError {
    fn from(err: InvalidEntryIndex) -> Self {
        match err.kind {
            BoundViolation::TooLarge =>
                RaftError::Store(RaftStorageError::Unavailable),
            BoundViolation::TooSmall =>
                RaftError::Store(RaftStorageError::Compacted),
        }
    }
}

impl Display for InvalidEntryIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Entry index was {}. Valid range [{}, {}), got {}.\n{}",
               self.kind,
               self.first_index,
               self.last_index,
               self.invalid_index,
               self.backtrace)
    }
}

impl PartialEq for InvalidEntryIndex {
    fn eq(&self, other: &InvalidEntryIndex) -> bool {
        self.kind == other.kind
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum BoundViolation {
    TooLarge,
    TooSmall,
}

impl Display for BoundViolation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match self {
            BoundViolation::TooLarge => "too large",
            BoundViolation::TooSmall => "too small",
        };
        write!(f, "{}", msg)
    }
}

impl Default for BoundViolation {
    fn default() -> Self {
        BoundViolation::TooSmall
    }
}
