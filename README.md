# bluez-rs
A library for controlling Bluetooth on Linux.

[![crates.io](https://img.shields.io/crates/v/bluez.svg?style=for-the-badge)](https://crates.io/crates/bluez)
[![crates.io](https://img.shields.io/crates/l/bluez.svg?style=for-the-badge)](https://github.com/laptou/bluez-rs/blob/master/LICENSE)

[Documentation](https://docs.rs/bluez)
[Examples](https://github.com/laptou/bluez-rs/blob/master/src/example/)

Some of the examples require elevated permissions. For example, to run the `discover` example, clone this repository, `cargo build`, and then `sudo target/debug/discover`.
`sudo` is necessary because many of the functions of this crate are not possible without the `CAP_NET_RAW` capability.

## License
This project is licensed under the MIT license.

### Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in bluez by you, shall be licensed as MIT, without any additional terms or conditions.
