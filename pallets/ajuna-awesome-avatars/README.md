# Ajuna Network Awesome Avatars Pallet

The Ajuna Awesome Avatars Pallet provides necessary extrinsics and storage for the collectible game
[AAA](https://aaa.ajuna.io). It allows:

- an organizer to manage the game and its seasons with various configurations
- players to obtain new avatars via minting, forging and trading
- players to trade avatars via setting / removing price for their avatars and buying others
- players to upgrade storage to hold more avatars

## Overview

The pallet must be initialized with a root call to set an account to act as an organizer.
The organizer can then set seasons with parameters to control various aspects of the game such as
the name, description and duration of a season as well as probabilities that affect forging
algorithm. When the network's block number reaches that of a season start, the season becomes active
and season-specific avatars can be obtained, which will no longer be available once the season
finishes. Avatars from previous seasons are available for trade if their owners are willing to sell.

An optional requirement for the pallet is an account to act as a season treasurer. Each season can
optionally have an associated treasurer who can claim the season's treasury once the season
finishes. It can be used as rewards for accounts who have contributed to a particular season.

## Reference Docs

You can view the reference docs for this pallet by running:

```
cargo doc --open
```
