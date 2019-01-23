# REOSC Node [![Telegram](https://img.shields.io/badge/telegram-join%20chat-blue.svg)](https://t.me/REOSC) [![Discord](https://img.shields.io/badge/discord-join%20chat-blue.svg)](https://discord.gg/CDQk9RR) [![License](https://img.shields.io/badge/license-GPL%20v3-green.svg)](https://www.gnu.org/licenses/gpl-3.0.en.html)

![REOSC Logo](./reosc.png)

## REOSC Mainnet node client

<p align="center"><strong><a href="https://github.com/REOSC/mn/releases/latest">» Download the latest release «</a></strong></p>

Welcome to the REOSC Node source code repository! This software enables businesses to rapidly build and deploy high-performance and high-security blockchain-based applications.

**Built for mission-critical use**: Miners, service providers, and exchanges need fast synchronisation and maximum uptime. REOSC Node provides the core infrastructure essential for speedy and reliable services.

- Clean, modular codebase for easy customisation
- Advanced CLI-based client
- Minimal memory and storage footprint
- Synchronise in hours, not days with Warp Sync
- Modular for light integration into your service or product
- Designed for scalable BFT - DPOS consensus blockchain platform with millions of transactions per second support
- Smart contract / Decentralized application platform powered by Web Assembly & EVM
- Designed for Inter Blockchain Communication (IBC) with EOSIO platfotm
- Designed for Decentralized Ethereum side-chain bridge

## Technical Overview

REOSC Node's goal is to be the fastest, lightest, and most secure REOSC client. We are developing REOSC using the sophisticated and cutting-edge **Rust programming language**. REOSC Node is licensed under the GPLv3 and can be used for all your REOSC needs.

By default, REOSC Node runs a JSON-RPC HTTP server on port `:8545` and a Web-Sockets server on port `:8546`. This is fully configurable and supports a number of APIs.

If you run into problems while using REOSC Node, check out the [wiki for documentation](https://wiki.parity.io/),
feel free to [file an issue in this repository](https://github.com/REOSC/reosc-eosc/issues/new), or hop on our [Discord](https://discord.gg/CDQk9RR) chat room to ask a question. We are glad to help! **For security-critical issues**, please refer to the security policy outlined in [SECURITY.md](SECURITY.md).

## Build Dependencies

REOSC Node requires **Rust version 1.30.x** to build.

We recommend installing Rust through [rustup](https://www.rustup.rs/). If you don't already have `rustup`, you can install it like this:

- Linux:
  ```bash
  $ curl https://sh.rustup.rs -sSf | sh
  ```

  REOSC Node also requires `gcc`, `g++`, `libudev-dev`, `pkg-config`, `file`, `make`, and `cmake` packages to be installed.

- OSX:
  ```bash
  $ curl https://sh.rustup.rs -sSf | sh
  ```

  `clang` is required. It comes with Xcode command line tools or can be installed with homebrew.

- Windows
  Make sure you have Visual Studio 2015 with C++ support installed. Next, download and run the `rustup` installer from
  https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe, start "VS2015 x64 Native Tools Command Prompt", and use the following command to install and set up the `msvc` toolchain:
  ```bash
  $ rustup default stable-x86_64-pc-windows-msvc
  ```

Once you have `rustup` installed, then you need to install:
* [Perl](https://www.perl.org)
* [Yasm](https://yasm.tortall.net)

Make sure that these binaries are in your `PATH`. After that, you should be able to build REOSC Node from source.

## Build REOSC Node from Source Code

For ubuntu >= 14.04

```bash
# Install dependencies from apt-get package manager
$ sudo apt-get -y install libudev-dev file cmake

# Install rustup if not installed
$ curl https://sh.rustup.rs -sSf | sh

# download REOSC Node code
$ git clone https://github.com/REOSC/reosc-node
$ cd reosc-node

# build in release mode
$ cargo build --release --features final
```

This produces an executable `reosc` in the `./target/release` subdirectory.

Note: if cargo fails to parse manifest try:

```bash
$ ~/.cargo/bin/cargo build --release
```

Note, when compiling a crate and you receive errors, it's in most cases your outdated version of Rust, or some of your crates have to be recompiled. Cleaning the repository will most likely solve the issue if you are on the latest stable version of Rust, try:

```bash
$ cargo clean
```

## Start REOSC Node

### Manually

To start REOSC Node manually, just run

```bash
$ ./target/release/reosc
```

so REOSC Node begins syncing the REOSC blockchain.

## About REOSC Node

REOSC Node is based off [Parity-ethereum](https://github.com/paritytech/parity-ethereum) Client for REOSC use.

It is a fork of Parity and will be maintained by REOSC team.

If you have any questions about our node please feel free to ask on telegramm channel.
