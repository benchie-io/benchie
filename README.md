# benchie

[![Build Status](https://img.shields.io/github/workflow/status/benchie-io/benchie/CI)](https://github.com/benchie-io/benchie/actions)
[![Crate](https://img.shields.io/crates/v/benchie.svg)](https://crates.io/crates/benchie)
[![API](https://docs.rs/benchie/badge.svg)](https://docs.rs/benchie)
![Rust Version](https://img.shields.io/badge/Rust-v1.60.0-yellow)
![Platform](https://img.shields.io/badge/platform-linux%20%7C%20macos%20%7C%20windows-brightgreen)
[![License](https://img.shields.io/crates/l/benchie)](https://github.com/benchie-io/benchie/blob/master/LICENSE)

### Usage

#### Binary

Once Rust is installed (see step 1 in "Toolchain Setup"), you can easily install the latest version of benchie with:
```
$ cargo install benchie --locked
$ benchie ./program --option 4
```

#### Library
Usage

Add this to your Cargo.toml:
```
[dependencies]
benchie = "0"
```

### Toolchain Setup
benchie can be build and tested on all major platforms.

1. Bootstrap Rust v1.60.0 from [https://rustup.rs](https://rustup.rs) and make sure:
- you install it with one of the supported host triples and
- add it to your path

### Build and Test from Source
1. Test your toolchain setup by compiling benchie:
```
$ cargo build --locked
```
2. Execute tests:
```
$ cargo test --locked
```

## License

Licensed under the [MIT](LICENSE) license.
