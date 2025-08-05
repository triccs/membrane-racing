// Trainer Contract Architecture

## Purpose

The Trainer contract handles all on-chain Q-learning logic. It updates Q-tables for car agents based on race results and reward signals. It is the sole authority responsible for learning, reward shaping, and training statistics. The contract now includes a simplified training abstraction with configurable parameters and state storage.

## Responsibilities

* Store and update Q-values per (car_id, state_hash)
* Support different reward types (`RewardType` enum)
* Enforce training rules: learning rate, reward shaping, max Q-values
* Track the number of training updates per car
* Record win/loss per track per car
* **NEW**: Execute configurable training sessions with state storage
* **NEW**: Support reward templates and custom reward parameters
* **NEW**: Provide fallback to contract constants for user parameters

## Core State

* `Q_TABLE`: `(car_id, state_hash) -> [i32; 5]` action values
* `TRAINING_STATS`: `car_id -> u32` number of updates
* `TRACK_RESULTS`: `(car_id, track_id) -> (u32 wins, u32 losses)`
* **NEW**: `ACTIVE_TRAINING_SESSIONS`: `car_id -> TrainingSession` active training sessions

## Core Messages

### Execute

* `BatchUpdateQValues { updates: Vec<QUpdate> }`
* **NEW**: `StartTraining { training_request: TrainingRequest }`

### Query

* `GetCarTrainingStats { car_id }` → number of training updates
* `GetTrackResults { car_id, track_id }` → (wins, losses)
* `GetQValue { car_id, state_hash }` → Q-values for all actions
* `GetTrainingProgress { car_id }` → training progress metrics
* `GetAntiStuckMetrics { car_id }` → anti-stuck performance metrics
* **NEW**: `GetTrainingSession { car_id }` → current training session
* `GetRewardTemplates {}` → available reward templates

## Training Request Structure

### TrainingRequest
- `car_id`: String - Unique identifier for the car being trained
- `track_id`: Option<String> - Track ID (optional, uses default if not specified)
- `training_config`: Option<TrainingConfig> - Training parameters (falls back to contract constants)
- `reward_config`: Option<RewardConfig> - Reward parameters (falls back to contract constants)
- `training_type`: TrainingType - Type of training strategy to use

### TrainingConfig
- `training_rounds`: u32 - Number of training rounds
- `steps_per_round`: u32 - Number of steps per round
- `exploration_rate`: f32 - Rate of exploration vs exploitation (0.0-1.0)
- `reward_multiplier`: f32 - Multiplier for reward values
- `stuck_threshold`: u32 - Consecutive non-forward actions before forcing forward
- `force_forward_after`: u32 - Consecutive forward actions before allowing exploration
- `use_progressive_exploration`: bool - Whether to use progressive exploration
- `initial_exploration`: Option<f32> - Initial exploration rate for progressive learning
- `final_exploration`: Option<f32> - Final exploration rate for progressive learning
- `enable_anti_stuck`: bool - Whether to enable anti-stuck mechanisms
- `track_metrics`: bool - Whether to track detailed metrics

### RewardConfig
- `Template(RewardTemplate)` - Use a predefined reward template
- `Custom(CustomRewardParams)` - Use custom reward parameters

## Reward Templates

### AntiStuck Template
- Focused on preventing cars from getting stuck in local optima
- Parameters: `forward_reward`, `stuck_penalty`, `exploration_bonus`, `anti_stuck_multiplier`

### Speed Template
- Optimized for fast track completion
- Parameters: `forward_reward`, `speed_bonus`, `slow_penalty`, `speed_multiplier`

### Conservative Template
- Safe and steady approach to racing
- Parameters: `forward_reward`, `risk_penalty`, `safety_bonus`, `conservative_multiplier`

### Aggressive Template
- High-risk, high-reward racing strategy
- Parameters: `forward_reward`, `aggressive_bonus`, `conservative_penalty`, `aggressive_multiplier`

### Balanced Template
- Well-rounded approach to racing
- Parameters: `forward_reward`, `exploration_bonus`, `stuck_penalty`, `balanced_multiplier`

## Training Types

* `AntiStuck`: Anti-stuck focused training
* `Progressive`: Progressive learning with decreasing exploration
* `Enhanced`: Enhanced training with high rewards
* `Balanced`: Balanced training approach
* `Custom`: Custom training with user-defined parameters

## Contract Constants (Fallback Values)

* `DEFAULT_EXPLORATION_RATE: f32 = 0.3`
* `DEFAULT_REWARD_MULTIPLIER: f32 = 1.5`
* `DEFAULT_STUCK_THRESHOLD: u32 = 2`
* `DEFAULT_FORCE_FORWARD_AFTER: u32 = 3`

## Training Session State

### TrainingSession
- `car_id`: String - Car ID being trained
- `track_id`: String - Track ID being used for training
- `training_config`: TrainingConfig - Training configuration
- `reward_config`: RewardConfig - Reward configuration
- `training_type`: TrainingType - Training type
- `current_round`: u32 - Current training round
- `total_rounds`: u32 - Total rounds completed
- `start_time`: u64 - Training start timestamp

## Reward Formula

`Q(s,a) = Q(s,a) + α × [reward + γ max Q(s',a') - Q(s,a)]`

* Learning rate: `ALPHA = 0.1`
* Discount factor: `GAMMA = 0.9`
* Clamped to `[-MAX_Q_VALUE, MAX_Q_VALUE]`

## StartTraining Flow

1. **Validation**: Check if training session already active for car
2. **Fallback Application**: Apply contract constants where user parameters are missing
3. **Session Creation**: Create and save training session to state
4. **Training Execution**: Execute training rounds based on configuration
5. **State Updates**: Update training session progress in state
6. **Cleanup**: Remove training session from state when complete

## Notes

* `next_state_hash` is optional depending on reward type.
* Reward logic is centralized here to keep simulation logic lightweight.
* Supports both single and batch training updates.
* **NEW**: Training sessions are saved in state during training.
* **NEW**: Users can specify custom parameters or use reward templates.
* **NEW**: Contract constants provide sensible defaults for missing parameters.
* **NEW**: Training progress is tracked and queryable through state.
