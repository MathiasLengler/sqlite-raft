use error::entries::NonSequentialEntryPair;
use failure::Backtrace;
use raft;
use raft::Error as RaftError;
use raft::StorageError as RaftStorageError;
use rusqlite;
use self::index::InvalidEntryIndex;
use std::result;

pub mod index;
pub mod entries;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "{}", _0)]
    Rusqlite(#[cause] rusqlite::Error, Backtrace),
    #[fail(display = "{}", _0)]
    Raft(#[cause] raft::Error, Backtrace),
    #[fail(display = "{}", _0)]
    InvalidEntryIndex(InvalidEntryIndex),
    #[fail(display = "{}", _0)]
    NonSequentialEntryPair(NonSequentialEntryPair),
}

impl From<rusqlite::Error> for Error {
    fn from(err: rusqlite::Error) -> Self {
        Error::Rusqlite(err, Backtrace::new())
    }
}

impl From<raft::Error> for Error {
    fn from(err: raft::Error) -> Self {
        Error::Raft(err, Backtrace::new())
    }
}

impl From<InvalidEntryIndex> for Error {
    fn from(err: InvalidEntryIndex) -> Self {
        Error::InvalidEntryIndex(err)
    }
}

impl From<NonSequentialEntryPair> for Error {
    fn from(err: NonSequentialEntryPair) -> Self {
        Error::NonSequentialEntryPair(err)
    }
}

// TODO: replace eprintln with trace
impl From<Error> for RaftError {
    fn from(err: Error) -> Self {
        match err {
            Error::Rusqlite(err, _backtrace) => {
//                eprintln!("{}", backtrace);
                RaftError::Store(RaftStorageError::Other(Box::new(err)))
            }
            Error::Raft(err, _backtrace) => {
//                eprintln!("{}", backtrace);
                err
            }
            Error::InvalidEntryIndex(err) => {
//                eprintln!("{}", err);
                err.into()
            }
            Error::NonSequentialEntryPair(err) => {
                err.into()
            }
        }
    }
}

impl PartialEq for Error {
    fn eq(&self, other: &Error) -> bool {
        use self::Error::*;

        match (self, other) {
            (InvalidEntryIndex(err), InvalidEntryIndex(other_err)) => err == other_err,
            (NonSequentialEntryPair(err), NonSequentialEntryPair(other_err)) => err == other_err,
            (Raft(err, _), Raft(other_err, _)) => err == other_err,
            _ => false
        }
    }
}
