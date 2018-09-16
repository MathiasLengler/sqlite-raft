//! # TODO
//!
//! ## Sanitize input using sqlpop:
//! match SELECT / INSERT
//! inline random()/etc.

#[macro_use]
extern crate failure;
extern crate protobuf;
extern crate rusqlite;
extern crate serde;
#[macro_use]
extern crate serde_derive;

pub mod connection;
pub mod error;
pub mod parameter;
pub mod query;
pub mod execute;
pub mod proto;
pub mod request;
mod value;

// TODO: move to modules (command?)

// TODO: rename crate
// TODO: Naming: "Command" trait -> "Request" trait

