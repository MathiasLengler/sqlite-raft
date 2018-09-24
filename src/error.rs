use failure::Backtrace;
use rusqlite;
use std::result;
use raft;
use raft::Error as RaftError;
use raft::StorageError as RaftStorageError;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "{}", _0)]
    Rusqlite(#[cause] rusqlite::Error, Backtrace),
    #[fail(display = "{}", _0)]
    Raft(#[cause] raft::Error, Backtrace)
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

impl From<Error> for RaftError {
    fn from(err: Error) -> Self {
        match err {
            Error::Rusqlite(err, backtrace) => {
                eprintln!("{}", backtrace);
                RaftError::Store(RaftStorageError::Other(Box::new(err)))
            },
            Error::Raft(err, backtrace) => {
                eprintln!("{}", backtrace);
                err
            },
        }
    }
}