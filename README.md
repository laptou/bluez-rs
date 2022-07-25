# bluez-rs
A library for controlling Bluetooth on Linux.

[![crates.io](https://img.shields.io/crates/v/bluez.svg?style=for-the-badge)](https://crates.io/crates/bluez)
[![crates.io](https://img.shields.io/crates/l/bluez.svg?style=for-the-badge)](https://github.com/laptou/bluez-rs/blob/master/LICENSE)

[Documentation](https://docs.rs/bluez)
[Examples](https://github.com/laptou/bluez-rs/tree/master/examples)

Some of the examples require elevated permissions. For example, to run the `discover` example, clone this repository, `cargo build --example discover`, then `sudo setcap cap_net_admin+ep target/debug/examples/discover`, then `target/debug/examples/discover`. Many of the functions of this crate are not possible without the `CAP_NET_ADMIN` capability.

## License
This project is licensed under the MIT license.

### Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in bluez by you, shall be licensed as MIT, without any additional terms or conditions.
