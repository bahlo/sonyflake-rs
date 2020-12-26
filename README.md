# sonyflake-rs

[![CI](https://github.com/bahlo/sonyflake-rs/workflows/CI/badge.svg)](https://github.com/bahlo/sonyflake-rs/actions?query=workflow%3ACI)
[![Audit](https://github.com/bahlo/sonyflake-rs/workflows/Audit/badge.svg)](https://github.com/bahlo/sonyflake-rs/actions?query=workflow%3AAudit)


A Rust port of [Sonyflake](https://github.com/sony/sonyflake), which is a distributed unique ID generator inspired by [Twitter's Snowflake](https://blog.twitter.com/2010/announcing-snowflake).

Since Sonyflake focuses on lifetime and performance on many host/core environments, it has a different bit assignment from Snowflake:

```
39 bits for time in units of 10 msec
 8 bits for a sequence number
16 bits for a machine id
```

These are the advantages and disadvantages of Sonyflake vs. Snowflake:

* The lifetime (174 years) is longer than that of Snowflake (69 years)
* It can work in more distributed machines (2^16) than Snowflake (2^10)
* It can generate 2^8 IDs per 10 msec at most in a single machine/thread (slower than Snowflake)

## Install

Add the following to your `Cargo.toml`:
```toml
[dependencies]
sonyflake = "0.1.0"
```
