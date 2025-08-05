// Project Architecture Overview: On-Chain RL Racing Game

## Vision

Create a fully on-chain racing game where players own AI car NFTs. Cars learn to race using reinforcement learning (Q-learning), and players can:

* Pay to train their models
* Enter weekly tournaments
* View past race histories
* Bet on races (in the future once randomness is added)
* **NEW**: Use advanced anti-stuck training strategies to improve car performance
* **NEW**: Monitor training progress and anti-stuck metrics

All training, racing, and logic is deterministic and verifiable on-chain.

## Core Contracts & Responsibilities

### 1. CarNFT

* CW721 NFT contract
* Stores unique car ID & Q-table (state-action values)
* Exposes Q-table via query
* Does NOT do training

### 2. Trainer

* Applies Q-learning logic using race results
* Handles all updates to Q-tables
* Supports different reward types
* Tracks stats: update count, win/loss per track
* **NEW**: Executes advanced anti-stuck training strategies
* **NEW**: Supports multiple training approaches (Enhanced, Progressive, Anti-Stuck, Smart)
* **NEW**: Tracks training progress and anti-stuck metrics
* **NEW**: Provides configurable training parameters and strategies

### 3. RaceEngine

* Executes simulation logic deterministically
* Loads track, car positions, and Q-tables
* Computes best actions and movement per tick
* Handles collisions, stick tiles, finish conditions
* Sends updates to Trainer

### 4. TrackManager

* Stores grid-based race tracks
* Each tile has a type (boost, slow, wall, stick, etc.)
* Each tile also stores progress_towards_finish
* Exposes full track layout via query

### 5. Tournament

* Weekly tournament organizer
* Selects car participants
* Uses RaceEngine to simulate tournament matches
* Tracks brackets and standings

## Contract Interactions

* `RaceEngine` queries:

  * Q-tables from `CarNFT`
  * Track layout from `TrackManager`

* `RaceEngine` sends:

  * QUpdates to `Trainer`
  * TrackResults to `Trainer`

* `Tournament` calls:

  * `RaceEngine` for races

* **NEW**: Players can directly call `Trainer` for advanced training:

  * `EnhancedTraining` for anti-stuck focused learning
  * `ProgressiveLearning` for controlled exploration
  * `AntiStuckTraining` for specific stuck prevention
  * `SmartTraining` for strategy-based training

## Q-Table Flow

1. Player pays to train a car (via RaceEngine contract)
2. RaceEngine simulates race
3. Trainer updates Q-table and training stats
4. Q-table is stored in `CarNFT` via `Trainer`
5. **NEW**: Players can use advanced training methods directly on Trainer
6. **NEW**: Training progress and metrics are tracked and queryable

## Anti-Stuck Training Features

### Training Strategies
* **Enhanced Training**: Configurable exploration with reward multipliers
* **Progressive Learning**: Decreasing exploration rate over time
* **Anti-Stuck Training**: Specific stuck prevention with thresholds
* **Smart Training**: Strategy-based training with multiple approaches

### Training Strategy Types
* `Random`: High exploration (80%) for broad learning
* `Guided`: Balanced exploration (40%) for focused learning
* `AntiStuck`: Moderate exploration (60%) with stuck prevention
* `Progressive`: Decreasing exploration rate over time
* `Enhanced`: Low exploration (30%) with high reward multipliers

### Metrics and Monitoring
* Training progress tracking
* Anti-stuck performance metrics
* Learning efficiency monitoring
* Stuck prevention rate analysis
* Q-value diversity tracking

## Notes

* All training is on-chain
* No randomness; non-determinism comes from car-to-car collision logic
* Race outcomes are auditable, re-runnable, and transparent
* **NEW**: Anti-stuck strategies prevent cars from getting stuck in local optima
* **NEW**: Training methods are designed to improve overall racing performance
* **NEW**: Comprehensive metrics allow for training optimization

## Goals

* Enable fully on-chain AI agents
* Make training participatory and paid
* Reward well-trained agents in tournaments
* **NEW**: Provide advanced training tools for better car performance
* **NEW**: Prevent training stagnation through anti-stuck strategies
* **NEW**: Enable data-driven training optimization through metrics
