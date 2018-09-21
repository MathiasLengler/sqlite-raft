use rusqlite;
use std::result;
use failure::Backtrace;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "{}", _0)]
    Rusqlite(#[cause] rusqlite::Error, Backtrace),
}

impl From<rusqlite::Error> for Error {
    fn from(err: rusqlite::Error) -> Self {
        Error::Rusqlite(err, Backtrace::new())
    }
}
