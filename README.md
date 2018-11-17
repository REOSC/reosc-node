# EOS Classic Node [![Build Status](https://travis-ci.org/eosclassic/node-eosc.svg?branch=master)](https://travis-ci.org/eosclassic/node-eosc) [![Telegram](https://img.shields.io/badge/telegram-join%20chat-blue.svg)](https://t.me/eoscio) [![Discord](https://img.shields.io/badge/discord-join%20chat-blue.svg)](https://discord.gg/EEzNEEu) [![License](https://img.shields.io/badge/license-GPL%20v3-green.svg)](https://www.gnu.org/licenses/gpl-3.0.en.html)

![EOS Classic Logo](./eosc-logo.png)

## Home of POW + DPOS engine implementation (EOS Classic Mainnet node client)

<p align="center"><strong><a href="https://github.com/eosclassic/node-eosc/releases/latest">» Download the latest release «</a></strong></p>

Welcome to the EOS Classic Node source code repository! This software enables businesses to rapidly build and deploy high-performance and high-security blockchain-based applications.

**Built for mission-critical use**: Miners, service providers, and exchanges need fast synchronisation and maximum uptime. EOS Classic Node provides the core infrastructure essential for speedy and reliable services.

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

EOS Classic Node's goal is to be the fastest, lightest, and most secure EOS Classic client. We are developing EOS Classic using the sophisticated and cutting-edge **Rust programming language**. EOS Classic Node is licensed under the GPLv3 and can be used for all your EOS Classic needs.

By default, EOS Classic Node runs a JSON-RPC HTTP server on port `:8545` and a Web-Sockets server on port `:8546`. This is fully configurable and supports a number of APIs.

If you run into problems while using EOS Classic Node, check out the [wiki for documentation](https://wiki.parity.io/), feel free to [file an issue in this repository](https://github.com/eosclassic/node-eosc/issues/new), or hop on our [Discord](https://discord.gg/EEzNEEu) chat room to ask a question. We are glad to help! **For security-critical issues**, please refer to the security policy outlined in [SECURITY.md](SECURITY.md).

## One-way EOS Classic node install script

Install EOS Classic Node by the following command

    $ curl -sL https://raw.githubusercontent.com/eosclassic/scripts/master/get-node-eosc.sh | sudo -E bash -

You can start eosc node by the following command, `eosc --chain=eosc` or just start with serviced `sudo systemctl start eosclassic`

## Build Dependencies

EOS Classic Node requires **Rust version 1.29.x** to build.

We recommend installing Rust through [rustup](https://www.rustup.rs/). If you don't already have `rustup`, you can install it like this:

- Linux:
  ```bash
  $ curl https://sh.rustup.rs -sSf | sh
  ```

  EOS Classic Node also requires `gcc`, `g++`, `libudev-dev`, `pkg-config`, `file`, `make`, and `cmake` packages to be installed.

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

Make sure that these binaries are in your `PATH`. After that, you should be able to build EOS Classic Node from source.

## Build EOS Classic Node from Source Code

For ubuntu >= 14.04

```bash
# Install dependencies from apt-get package manager
$ sudo apt-get -y install libudev-dev file cmake

# Install rustup if not installed
$ curl https://sh.rustup.rs -sSf | sh

# download EOS Classic Node code
$ git clone https://github.com/eosclasssic/node-eosc
$ cd node-eosc

# build in release mode
$ cargo build --release --features final
```

For windows (to-do)

This produces an executable `eosc` in the `./target/release` subdirectory.

Note: if cargo fails to parse manifest try:

```bash
$ ~/.cargo/bin/cargo build --release
```

Note, when compiling a crate and you receive errors, it's in most cases your outdated version of Rust, or some of your crates have to be recompiled. Cleaning the repository will most likely solve the issue if you are on the latest stable version of Rust, try:

```bash
$ cargo clean
```

## Start EOS Classic Node

### Manually

To start EOS Classic Node manually, just run

```bash
$ ./target/release/eosc
```

so EOS Classic Node begins syncing the EOS Classic blockchain.

### Using `systemd` service file

To start EOS Classic Node as a regular user using `systemd` init:

1. Download `https://raw.githubusercontent.com/eosclassic/scripts/master/node-eosc/eosclassic.service` to your
`systemd` user directory (usually `~/.config/systemd/user`).
2. To configure EOS Classic Node, write a `/etc/parity/config.toml` config file, see [Configuring EOS Classic Node](https://paritytech.github.io/wiki/Configuring-Parity) for details.

## About EOS Classic Node

EOS Classic Node is based off [Parity-ethereum](https://github.com/paritytech/parity-ethereum) Client for EOS Classic use

It is a fork of Parity and will be maintained by EOS Classic team along with our upcoming C++ node implementation

If you have any questions about our node please feel free to ask on discord channel
