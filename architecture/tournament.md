// Tournament Contract Architecture

## Purpose

The Tournament contract manages weekly race competitions among eligible car NFTs. It determines selection, scoring, and progression rules.

## Responsibilities

* Select participants from all available cars
* Organize bracket-style or leaderboard-based races
* Interface with RaceEngine to simulate matches
* Record and expose results

## Core Components

* `TOURNAMENT_STATE`: stores current tournament metadata
* `PARTICIPANTS`: list of car\_ids in current week
* `TOURNAMENT_RESULTS`: final rankings

## Core Flow

1. **StartTournament**

   * Select eligible cars (random, top-trained, etc.)
   * Save snapshot

2. **SimulateRounds**

   * Use RaceEngine to simulate matches between brackets

3. **EndTournament**

   * Record winner(s)
   * Emit final standings

## Core Messages

### Execute

* `StartTournament { criteria }`
* `RunNextRound {}`
* `EndTournament {}`

### Query

* `CurrentBracket` → who’s racing
* `TournamentResults` → full final rankings
* `IsParticipant { car_id }` → check inclusion

## Notes

* This contract does not do training.
* RaceEngine handles simulation; Trainer contract unaffected.
* Could allow betting in the future.
