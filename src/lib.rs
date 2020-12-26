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
//! let mut sf = sonyflake::Builder::new()
//!     .finalize()
//!     .expect("Could not construct Sonyflake");
//! let next_id = sf.next_id().expect("Could not get next id");
//! println!("{}", next_id);
//! ```
//!
//! [Sonyflake]: https://github.com/sony/sonyflake

pub mod builder;
pub mod error;
pub mod sonyflake;

pub use crate::builder::Builder;
pub use crate::sonyflake::Sonyflake;

#[cfg(test)]
mod tests {
    use crate::Builder;

    #[test]
    fn next_id() {
        let mut sf = Builder::new()
            .finalize()
            .expect("Could not construct Sonyflake");
        assert!(sf.next_id().is_ok());
    }
}
