# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- Make `pnet` a feature (enabled by default). This can be turned off if there's
  platform problems or people want to bring their own function to generate a
  machine id and want to save a dependency.
- Upgrade crate to Rust Edition 2024

## [0.3.0] - 2024-10-09

### Changed

- Use pnet datalink directly, only use necessary features of dependencies

### Fixed

- Replace depcreated `timestamp_nanos_opt` use

## [0.2.0] - 2023-06-22

### Changes

* Remove `&mut` requirement from `Sonfylake.next_id`
* Decompose now return a struct instead of a HashMap
* Upgrade crate to Rust Edition 2021
* Upgrade pnet to 0.33

## [0.1.2] - 2021-09-11

### Changed

- Bump pnet to 0.28 to mitigate RUSTSEC-2018-0015

## [0.1.1] - 2020-12-29

### Fixed

- `Error` is now `Send` + `Sync`

## [0.1.0] - 2020-12-29

This is the initial version.

[unreleased]: https://github.com/bahlo/sonyflake-rs/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/bahlo/sonyflake-rs/releases/tag/v0.3.0
[0.2.0]: https://github.com/bahlo/sonyflake-rs/releases/tag/v0.2.0
[0.1.2]: https://github.com/bahlo/sonyflake-rs/releases/tag/v0.1.2
[0.1.1]: https://github.com/bahlo/sonyflake-rs/releases/tag/v0.1.1
[0.1.0]: https://github.com/bahlo/sonyflake-rs/releases/tag/v0.1.0

