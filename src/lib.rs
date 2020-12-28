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
//! let mut sf = Sonyflake::new().unwrap();
//! let next_id = sf.next_id().unwrap();
//! println!("{}", next_id);
//! ```
//!
//! ## Concurrent use
//!
//! Sonyflake is threadsafe. `clone` it before moving to another thread:
//! ```
//! use sonyflake::Sonyflake;
//! use std::thread;
//!
//! let sf = Sonyflake::new().unwrap();
//!
//! let mut handles = vec![];
//! for _ in 0..10 {
//!     let mut sfc = sf.clone();
//!     handles.push(thread::spawn(move || {
//!         let _next_id = sfc.next_id().unwrap();
//!     }));
//! }
//!
//! for handle in handles {
//!     handle.join().unwrap();
//! }
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
