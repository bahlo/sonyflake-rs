//! A distributed unique ID generator inspired by [Twitter's Snowflake].
//!
//! This is a Rust implementation of the original [sony/sonyflake], which is written in Go.
//!
//! # Example
//!
//! ```
//! use sonyflake::Sonyflake;
//!
//! let mut sf = Sonyflake::new().unwrap();
//! let next_id = sf.next_id().unwrap();
//! println!("{}", next_id);
//! ```
//!
//! [sony/sonyflake]: https://github.com/sony/sonyflake
//! [Twitter's Snowflake]: https://blog.twitter.com/2010/announcing-snowflake
#![deny(warnings)]
#![deny(clippy::pedantic, clippy::unwrap_used)]
#![deny(missing_docs)]
#![doc(html_root_url = "https://docs.rs/sonyflake/*")]

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
pub struct ReadmeDoctests;

mod builder;
mod error;
mod sonyflake;
#[cfg(test)]
mod tests;

pub use crate::sonyflake::*;
pub use builder::*;
pub use error::*;
