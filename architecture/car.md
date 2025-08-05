// Car NFT Contract Architecture

## Purpose

The Car contract implements CW721 NFT logic to represent individually trainable car agents. Each car holds metadata and exposes a queryable interface for its Q-learning state.

## Responsibilities

* Mint and track unique car NFTs
* Store car metadata and ownership
* Support Q-learning integration by exposing Q-tables
* Allow querying full Q-table or individual state hashes

## Core State

* `CAR_INFO`: `car_id -> CarMetadata { owner, ... }`
* `Q_TABLE`: `(car_id, state_hash) -> [i32; 5]`

## Core Messages

### Execute (Standard CW721 + Extended)

* `Mint { owner, car_id, ... }`
* Optional: `UpdateCarMetadata`

### Query

* `GetQ { car_id, state_hash: Option<String> }` → full Q-table or one entry
* `OwnerOf { token_id }` → CW721
* `NftInfo { token_id }` → CW721
* `AllTokens {}` → CW721

## Notes

* Q-tables are *not* updated by this contract.
* Cars receive updates via the `Trainer`, not directly.
* Exposes clean prefix-based queries for full table access.
* Can be extended with visual traits, race stats, etc.
