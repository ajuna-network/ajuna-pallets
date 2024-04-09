# Ajuna Pallets

<p align="center">
  <a href="https://ajuna.io" target="_blank">
    <img src="docs/ajuna-banner.jpeg" alt="Ajuna Network Banner">
  </a>
</p>

[![Build](https://github.com/ajuna-network/ajuna-pallets/actions/workflows/check-pull-request.yml/badge.svg?branch=main)](https://github.com/ajuna-network/ajuna-pallets/actions/workflows/check-pull-request.yml)
[![codecov](https://codecov.io/gh/ajuna-network/ajuna-pallets/branch/main/graph/badge.svg?token=qRtKAiLsbG)](https://codecov.io/gh/ajuna-network/ajuna-pallets)

This repository contains the different FRAME pallets used in the Ajuna/Bajun ecosystem.

## Managing Dependencies
We use [psvm](https://github.com/paritytech/psvm) to manage substrate/polkadot dependencies.

```bash
# Install or update psvm. The available polkadot versions are hardcoded in the 
# psvm release. Hence, we need to update it regularly.
cargo install --git https://github.com/paritytech/psvm psvm

# Example: update to polkadot version 1.9.0
psvm -v "1.9.0"
```