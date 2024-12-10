#!/usr/bin/env bash

# Useful for checking individual crates for correctness, i.e., if they compile to wasm.

set -e

COMMAND=$1
shift

set -x

find . -name "Cargo.toml" | while read -r CARGO_TOML; do
  DIR=$(dirname "$CARGO_TOML")
  echo "Checking in directory: $DIR"

  if grep -q "\[features\]" "$CARGO_TOML" && grep -q "runtime-benchmarks" "$CARGO_TOML"; then
      echo "Feature 'runtime-benchmarks' found, adding this feature."
      cargo $COMMAND $@ --features runtime-benchmarks --manifest-path "$CARGO_TOML"
  else
      echo "Feature 'runtime-benchmarks' not found, running command without this feature"
      cargo $COMMAND $@ --manifest-path "$CARGO_TOML";
  fi
done