# Anti-Stuck Training Architecture

## Overview

The anti-stuck training system is designed to prevent AI cars from getting stuck in local optima during training. It implements multiple strategies to ensure cars learn effective racing behaviors and avoid repetitive, non-productive actions.

## Problem Statement

Traditional Q-learning can lead to cars getting "stuck" in suboptimal behaviors:
- Cars may repeatedly take the same non-forward actions
- Local optima can trap cars in inefficient movement patterns
- Lack of exploration can prevent discovery of better strategies
- Training can stagnate without proper anti-stuck mechanisms

## Anti-Stuck Strategies

### 1. Enhanced Training

**Purpose**: Provide configurable exploration with enhanced rewards to prevent stuck behavior.

**Key Features**:
- Configurable exploration rate and reward multiplier
- Enhanced reward structure with anti-stuck incentives
- Anti-stuck action selection with enhanced exploration
- Tracks optimal actions and stuck actions

**Parameters**:
- `training_rounds`: Number of training rounds
- `steps_per_round`: Steps per training round
- `exploration_rate`: Rate of exploration vs exploitation (0.0-1.0)
- `reward_multiplier`: Multiplier for reward values

**Strategy**:
- Early steps: prefer forward movement with some exploration
- Later steps: more exploration to prevent stuck
- Enhanced rewards for forward movement
- Configurable exploration rate

### 2. Progressive Learning

**Purpose**: Gradually reduce exploration while maintaining anti-stuck capabilities.

**Key Features**:
- Exploration rate that decreases over time
- Progressive reward structure
- Controlled exploration to prevent stuck
- Tracks learning progression

**Parameters**:
- `initial_exploration`: Starting exploration rate (0.0-1.0)
- `final_exploration`: Final exploration rate (0.0-1.0)
- `learning_rounds`: Number of learning rounds
- `steps_per_round`: Steps per learning round

**Strategy**:
- Starts with high exploration (initial_exploration)
- Gradually reduces to final_exploration
- Early steps: mostly forward with limited exploration
- Later steps: more exploration to prevent stuck

### 3. Anti-Stuck Training

**Purpose**: Specific stuck prevention with configurable thresholds.

**Key Features**:
- Specific stuck prevention logic
- Configurable stuck threshold and force forward parameters
- Tracks consecutive non-forward actions
- Forces forward movement when stuck

**Parameters**:
- `track_length`: Length of the track being trained on
- `training_rounds`: Number of training rounds
- `stuck_threshold`: Consecutive non-forward actions before forcing forward
- `force_forward_after`: Consecutive forward actions before allowing exploration

**Strategy**:
- Monitors consecutive non-forward actions
- Forces forward movement if stuck threshold exceeded
- Allows exploration after good forward progress
- Tracks forced forward moves and stuck actions

### 4. Smart Training

**Purpose**: Combine multiple strategies based on configurable training approach.

**Key Features**:
- Combines multiple strategies based on `TrainingStrategy` enum
- Supports Random, Guided, AntiStuck, Progressive, and Enhanced strategies
- Configurable track training parameters
- Adaptive training based on strategy type

**Training Strategy Types**:
- `Random`: High exploration rate (80%) for broad learning
- `Guided`: Balanced exploration (40%) for focused learning
- `AntiStuck`: Moderate exploration (60%) with stuck prevention
- `Progressive`: Decreasing exploration rate over time
- `Enhanced`: Low exploration (30%) with high reward multipliers

## Anti-Stuck Constants

```rust
const DEFAULT_EXPLORATION_RATE: f32 = 0.3;
const DEFAULT_REWARD_MULTIPLIER: f32 = 1.5;
const DEFAULT_STUCK_THRESHOLD: u32 = 2;
const DEFAULT_FORCE_FORWARD_AFTER: u32 = 3;
```

## Metrics and Monitoring

### Training Progress Metrics
- `total_rounds`: Total number of training rounds completed
- `current_round`: Current training round number
- `optimal_actions`: Number of optimal actions (forward movements)
- `stuck_actions`: Number of stuck actions (non-forward movements)
- `learning_efficiency`: Learning efficiency as percentage (0.0-100.0)
- `stuck_prevention_rate`: Stuck prevention rate as percentage (0.0-100.0)

### Anti-Stuck Metrics
- `avg_optimal_actions`: Average number of optimal actions per round
- `avg_stuck_actions`: Average number of stuck actions per round
- `learning_efficiency`: Overall learning efficiency as percentage
- `stuck_prevention_rate`: Overall stuck prevention rate as percentage
- `track_completion_rate`: Track completion rate as percentage
- `q_value_diversity`: Diversity of Q-values (number of different Q-values > 0)

## Track Training Configuration

```rust
pub struct TrackTrainingConfig {
    pub track_length: u32,
    pub early_stage_end: u32,
    pub middle_stage_end: u32,
    pub late_stage_end: u32,
    pub early_reward: i32,
    pub middle_reward: i32,
    pub late_reward: i32,
    pub finish_reward: i32,
}
```

## Implementation Details

### Random Function
Uses a simple deterministic random function for CosmWasm:
```rust
fn simple_random(step: u32, max: u32) -> u32 {
    (step * 1103515245 + 12345) % max
}

fn random_float(step: u32) -> f32 {
    (simple_random(step, 1000) as f32) / 1000.0
}
```

### Action Selection Logic
- Forward action (0) is preferred for optimal racing
- Non-forward actions (1-4) are used for exploration
- Stuck detection monitors consecutive non-forward actions
- Force forward mechanism prevents infinite stuck loops

### Reward Structure
- Enhanced rewards for forward movement
- Penalties for stuck behavior
- Stage-specific rewards based on track position
- Configurable reward multipliers

## Benefits

1. **Prevents Training Stagnation**: Anti-stuck strategies prevent cars from getting trapped in local optima
2. **Improves Learning Efficiency**: Better exploration-exploitation balance
3. **Enhances Racing Performance**: Cars learn more effective racing strategies
4. **Provides Monitoring**: Comprehensive metrics for training optimization
5. **Configurable**: Multiple strategies and parameters for different training needs

## Usage

Players can call anti-stuck training methods directly on the Trainer contract:

```rust
// Enhanced training with anti-stuck focus
ExecuteMsg::EnhancedTraining {
    car_id: "car_123",
    training_rounds: 30,
    steps_per_round: 20,
    exploration_rate: 0.4,
    reward_multiplier: 1.5,
}

// Progressive learning with decreasing exploration
ExecuteMsg::ProgressiveLearning {
    car_id: "car_123",
    initial_exploration: 0.8,
    final_exploration: 0.2,
    learning_rounds: 25,
    steps_per_round: 20,
}

// Anti-stuck training with specific thresholds
ExecuteMsg::AntiStuckTraining {
    car_id: "car_123",
    track_length: 20,
    training_rounds: 40,
    stuck_threshold: 3,
    force_forward_after: 5,
}

// Smart training with strategy selection
ExecuteMsg::SmartTraining {
    car_id: "car_123",
    track_config: TrackTrainingConfig { ... },
    training_strategy: TrainingStrategy::AntiStuck,
}
``` 