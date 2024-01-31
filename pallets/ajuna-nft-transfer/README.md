# Ajuna NFT-Transfer Pallet

The Ajuna NFT Transfer Pallet provides necessary functionalities to tokenize an arbitrary piece of
data that supports the SCALE codec into an appropriate NFT representation. It interfaces with the
non-fungible traits to support their arbitrary NFT standards and underlying storage solutions.

## Overview

The pallet must be initialized with a collection ID, created externally via `pallet-nfts`, to group
similar NFTs under the same collection. In order to store and recover NFTs, the `NftConvertible`
trait must be implemented by the objects of interest. When storing NFTs, the owners pay for the
associated deposit amount, which is fully refunded when the NFTs are recovered back into their
original form.

## Reference Docs

You can view the reference docs for this pallet by running:

```
cargo doc --open
```
