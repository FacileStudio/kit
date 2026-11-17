#!/usr/bin/env sh
set -eu

cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
if [ -f install.sh ]; then sh -n install.sh; fi
filet check .
