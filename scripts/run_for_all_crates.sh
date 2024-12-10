#!/usr/bin/env bash

# Useful for checking individual crates for correctness, i.e., if they compile to wasm.

set -e

COMMAND=$1
shift

set -x

for file in **/Cargo.toml; do
  if grep -q "\[features\]" "$file" && grep -q "runtime-benchmarks" "$file"; then
      echo "Feature 'runtime-benchmarks' found, adding this feature."
      cargo $COMMAND $@ --features runtime-benchmarks --manifest-path "$file"
  else
      echo "Feature 'runtime-benchmarks' not found, running command without this feature"
      cargo $COMMAND $@ --manifest-path "$file";
  fi
done