use failure::Fail;
use rusqlite;
use std::result;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "No queued parameters for SQLite command. At least one (empty) parameter list must be given.")]
    NoQueuedParameters,

    #[fail(display = "{}", _0)]
    Rusqlite(#[cause] rusqlite::Error),
}

impl From<rusqlite::Error> for Error {
    fn from(err: rusqlite::Error) -> Self {
        Error::Rusqlite(err)
    }
}