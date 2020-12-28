//! Sonyflake is a Rust implementation of the [Sonyflake] algorithm.
//!
//! ## Quickstart
//!
//! Add the following to your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! sonyflake = "0.1.0"
//! ```
//!
//! Use the library like this:
//!
//! ```
//! use sonyflake::Sonyflake;
//!
//! let mut sf = Sonyflake::new().expect("Could not construct Sonyflake");
//! let next_id = sf.next_id().expect("Could not get next id");
//! println!("{}", next_id);
//! ```
//!
//! [Sonyflake]: https://github.com/sony/sonyflake

mod builder;
mod error;
mod sonyflake;
#[cfg(test)]
mod tests;

pub use crate::sonyflake::*;
pub use builder::*;
pub use error::*;
