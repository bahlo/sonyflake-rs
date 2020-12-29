# sonyflake-rs

[![CI](https://github.com/bahlo/sonyflake-rs/workflows/CI/badge.svg)](https://github.com/bahlo/sonyflake-rs/actions?query=workflow%3ACI)
[![Audit](https://github.com/bahlo/sonyflake-rs/workflows/Audit/badge.svg)](https://github.com/bahlo/sonyflake-rs/actions?query=workflow%3AAudit)
[![crates.io](https://img.shields.io/crates/v/sonyflake.svg)](https://crates.io/crates/sonyflake)
[![docs.rs](https://docs.rs/sonyflake/badge.svg)](https://docs.rs/sonyflake/)

A distributed unique ID generator inspired by [Twitter's Snowflake](https://blog.twitter.com/2010/announcing-snowflake).

This is a Rust implementation of the original [sony/sonyflake](https://github.com/sony/sonyflake), which is written in Go.

## Install

Add the following to your `Cargo.toml`:
```toml
[dependencies]
sonyflake = "0.1"
```

## Quickstart

```rust
use sonyflake::Sonyflake;

let mut sf = Sonyflake::new().unwrap();
let next_id = sf.next_id().unwrap();
println!("{}", next_id);
```

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
