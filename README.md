[![crates.io](https://img.shields.io/crates/v/bluez.svg?style=for-the-badge)](https://crates.io/crates/bluez)
[![crates.io](https://img.shields.io/crates/l/bluez.svg?style=for-the-badge)](https://crates.io/crates/bluez)
# bluez-rs

This crate is an interface to the Linux Bluetooth API, Bluez that uses the socket API under the hood instead of the DBus API. This will allow very performant control of Bluetooth on Linux. However, this crate is not anywhere near complete yet.

## todo
- finish implementing all management commands and events
- create event loop to receive events that are not in response to a command
- create client to house event loop and socket in user-friendly API
