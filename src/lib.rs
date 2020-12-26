pub mod builder;
pub mod error;
pub mod sonyflake;

mod prelude {
    pub use crate::builder::SonyflakeBuilder;
    pub use crate::sonyflake::Sonyflake;
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn next_id() {
        let mut sf = SonyflakeBuilder::new()
            .finalize()
            .expect("Could not construct Sonyflake");
        assert!(sf.next_id().is_ok());
    }
}
