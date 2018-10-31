//! # TODO
//!
//! ## Sanitize input using sqlpop:
//! match SELECT / INSERT
//! inline random()/etc.

pub mod connection;
pub mod error;
pub mod parameter;
pub mod query;
pub mod execute;
pub mod proto;
pub mod request;
mod value;
