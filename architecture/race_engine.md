# RaceEngine Architecture Document

This document defines the architecture and design of the `RaceEngine` CosmWasm smart contract, optimized for implementation and reasoning by language models.

## Purpose

The `RaceEngine` contract is responsible for executing simulated races between AI-controlled cars whose behavior is driven by Q-learning. It supports both PvP and training use cases and outputs results for on-chain recording and reward logic.

---

## Core Concepts

### Cars

* Identified by `car_id: String`
* Controlled by a Q-table stored off-contract in the `Trainer` contract (prefetched at start of race)

### Tracks

* Identified by `track_id: String`
* Stored in `track_manager`
* A 2D grid (`layout: Vec<Vec<Tile>>`) where each tile has:

  * `tile_type`: Enum (Normal, Stick, Boost, Slow, Wall, Finish)
  * `progress_towards_finish: u32`

### Tiles and Effects

* `Normal`: No special effect
* `Stick`: Move allowed, but skips the next turn
* `Slow`: Always move only 1 tile forward
* `Boost`: Move 3 tiles forward if chosen
* `Wall`: Block movement completely
* `Finish`: Target for winning condition

### Simulation

* All cars act simultaneously each tick
* Collisions prevent movement for involved cars (handled after all intents are calculated)
* State per car includes:

  * Position `(x, y)`
  * `stuck: bool`
  * `finished: bool`
  * Steps taken

### Play-by-Play

* Each tick is recorded as a string in `play_by_play: Vec<String>`
* Used for replay/debugging

### Termination Conditions

* Max ticks (`MAX_TICKS = 100`)
* All cars have finished

---

## External Dependencies

* `CarNFT` contract:

  * Stores car metadata and unique ownership
  * Stores Q-tables in a prefixed format for training

* `TrackManager` contract:

  * Provides validated track layouts and tile data

* `Trainer` contract:

  * Handles Q-table updates
  * Accepts reward messages via `BatchUpdateQValues`
  * Accepts win/loss records via `RecordTrackResult`

---

## Key Functions

### `simulate_race(params: SimulateRaceParams)`

1. Load track and car IDs
2. Fetch Q-table for each car
3. Simulate tick-by-tick movement:

   * Compute car intents (actions)
   * Apply tile rules (Slow, Stick, Wall)
   * Detect and resolve collisions
   * Record play-by-play
   * Track steps and finish status
4. Determine winner by finish first or closest to finish
5. Send `BatchUpdateQValues` to Trainer contract
6. Send `RecordTrackResult` per car to Trainer
7. Save `RaceResult` to ring buffer
8. Emit attributes for result summary

### `query::GetRaceResult { race_id }`

Returns a `RaceResult` from the recent buffer.

### Upcoming Queries:

* `ListRecentRaces`: Show metadata of last N races
* `QueryWinnerStats`: Show total wins/losses per track

---

## State

### `RECENT_RACES: Item<Vec<RaceResult>>`

* Stores last `MAX_RECENT_RACES` race results (FIFO ring buffer)

### `RaceResult`

```rust
struct RaceResult {
  winner_ids: Vec<String>,
  rankings: Vec<(String, u32)>,
  play_by_play: Vec<String>
}
```

---

## Design Goals

* Deterministic for training
* Supports emergent randomness via car-to-car collisions
* Decouples track data and Q-table logic for modularity
* Future-friendly for betting/tournaments/game modes

---

## Suggested Enhancements

* Add `SimulateRaceParams.max_ticks` override
* Track race timestamps
* Add support for tournaments with brackets
* Include fuel system or energy to encourage efficient paths

---

## Compatibility Notes

* All simulations should be purely deterministic based on Q-tables and track state
* No RNG should be introduced directly
* All Q-table data should be prefetched to reduce gas
