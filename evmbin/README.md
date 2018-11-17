## evmbin

EVM implementation for Parity.

### Usage

```
EVM implementation for Parity.
  Copyright 2015-2018 Parity Technologies (UK) Ltd.

Usage:
    parity-evm state-test <file> [--json --std-json --only NAME --chain CHAIN]
    parity-evm stats [options]
    parity-evm stats-jsontests-vm <file>
    parity-evm [options]
    parity-evm [-h | --help]

Commands:
    state-test         Run a state test from a json file.
    stats              Execute EVM runtime code and return the statistics.
    stats-jsontests-vm Execute standard json-tests format VMTests and return
                       timing statistics in tsv format.

Transaction options:
    --code CODE        Contract code as hex (without 0x).
    --to ADDRESS       Recipient address (without 0x).
    --from ADDRESS     Sender address (without 0x).
    --input DATA       Input data as hex (without 0x).
    --gas GAS          Supplied gas as hex (without 0x).
    --gas-price WEI    Supplied gas price as hex (without 0x).

State test options:
    --only NAME        Runs only a single test matching the name.
    --chain CHAIN      Run only tests from specific chain.

General options:
    --json             Display verbose results in JSON.
    --std-json         Display results in standardized JSON format.
    --chain CHAIN      Chain spec file path.
    -h, --help         Display this message and exit.
```

## Parity Ethereum toolchain
_This project is a part of the Parity Ethereum toolchain._

- [evmbin](https://github.com/paritytech/parity-ethereum/blob/master/evmbin/) - EVM implementation for Parity Ethereum.
- [ethabi](https://github.com/paritytech/ethabi) - Parity Ethereum function calls encoding.
- [ethstore](https://github.com/paritytech/parity-ethereum/blob/master/ethstore/) - Parity Ethereum key management.
- [ethkey](https://github.com/paritytech/parity-ethereum/blob/master/ethkey/) - Parity Ethereum keys generator.
- [whisper](https://github.com/paritytech/parity-ethereum/blob/master/whisper/) - Implementation of Whisper-v2 PoC.
