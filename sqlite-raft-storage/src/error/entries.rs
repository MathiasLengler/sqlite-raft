use failure::Backtrace;
use failure::Fail;
use raft::eraftpb::Entry;
use raft::Error as RaftError;
use raft::StorageError as RaftStorageError;
use std::fmt;
use std::fmt::Display;

#[derive(Debug, Fail)]
pub struct NonSequentialEntryPair {
    pub incompatible_entry: Entry,
    pub previous_entry: Entry,
    pub cause: SequenceViolation,
    pub backtrace: Backtrace,
}

impl Display for NonSequentialEntryPair {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Entry {:?} is incompatible with previous entry {:?}: {}.",
               self.incompatible_entry,
               self.previous_entry,
               self.cause)
    }
}

impl From<NonSequentialEntryPair> for RaftError {
    fn from(err: NonSequentialEntryPair) -> Self {
        RaftError::Store(RaftStorageError::Other(Box::new(err.compat())))
    }
}

impl PartialEq for NonSequentialEntryPair {
    fn eq(&self, other: &NonSequentialEntryPair) -> bool {
        self.cause == other.cause
            && self.incompatible_entry == other.incompatible_entry
            && self.previous_entry == other.previous_entry
    }
}

#[derive(Debug, Fail, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum SequenceViolation {
    #[fail(display = "incompatible index")]
    IncompatibleIndex,
    #[fail(display = "decreasing term")]
    DecreasingTerm,
}
