#!/bin/bash

set -e # fail on any error
set -u # treat unset variables as error

cargo install cargo-audit
cargo audit
