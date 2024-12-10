#!/usr/bin/env bash

# Useful for checking individual crates for correctness, i.e., if they compile to wasm.

set -e

COMMAND=$1
shift

set -x

for file in **/Cargo.toml; do
	cargo $COMMAND $@ --manifest-path "$file";
done