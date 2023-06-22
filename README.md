# sonyflake-rs

[![Build](https://github.com/bahlo/sonyflake-rs/workflows/Build/badge.svg)](https://github.com/bahlo/sonyflake-rs/actions?query=workflow%3ABuild)
[![crates.io](https://img.shields.io/crates/v/sonyflake.svg)](https://crates.io/crates/sonyflake)
[![docs.rs](https://docs.rs/sonyflake/badge.svg)](https://docs.rs/sonyflake/)
[![License](https://img.shields.io/crates/l/sonyflake)](LICENSE-APACHE)

A distributed unique ID generator inspired by [Twitter's Snowflake](https://blog.twitter.com/2010/announcing-snowflake).

This is a Rust implementation of the original [sony/sonyflake](https://github.com/sony/sonyflake), which is written in Go.

A Sonyflake ID is composed of

- 39 bits for time in units of 10 msec
- 8 bits for a sequence number
- 16 bits for a machine id

## Install

Add the following to your `Cargo.toml`:
```toml
[dependencies]
sonyflake = "0.1"
```

## Quickstart

```rust
use sonyflake::Sonyflake;

let sf = Sonyflake::new().unwrap();
let next_id = sf.next_id().unwrap();
println!("{}", next_id);
```

## Benchmarks

Benchmarks were run on a MacBook Pro (15-inch, 2017) with a 2,8GHz i7 and 16 GB memory.
Run them yourself with `cargo bench`.

```benchmark
test bench_decompose ... bench:       1,066 ns/iter (+/- 132)
test bench_new       ... bench:     738,129 ns/iter (+/- 318,192)
test bench_next_id   ... bench:      37,390 ns/iter (+/- 499)
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
