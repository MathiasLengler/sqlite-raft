// TODO:
// replace with include!(concat!(env!("OUT_DIR"), "/sqlite_requests.rs"))
// when https://github.com/intellij-rust/intellij-rust/issues/1908 is resolved

mod sqlite_requests;
pub use self::sqlite_requests::*;
