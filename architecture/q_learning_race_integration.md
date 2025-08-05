# Q-Learning Race Integration Architecture

## Overview

The race engine contract now automatically triggers Q-learning updates after each race simulation. This creates a complete feedback loop where:

1. **Race Simulation**: Cars race on tracks using their current Q-values
2. **Action Recording**: All car actions are recorded during the race
3. **Reward Calculation**: Rewards are calculated based on race performance
4. **Q-Value Updates**: The trainer contract updates Q-values based on race results
5. **Learning**: Cars improve their performance for future races

## Flow Diagram

```
Race Engine → Simulate Race → Record Actions → Calculate Rewards → Trainer Contract → Update Q-Values
     ↓              ↓              ↓              ↓              ↓              ↓
  Car Actions   State Hashes   Race Results   Reward Types   Batch Updates   Learning
```

## Key Components

### 1. Action History Tracking

**Location**: `contracts/race-engine/src/contract.rs`

**CarState Structure**:
```rust
struct CarState {
    car_id: String,
    x: i32,
    y: i32,
    stuck: bool,
    finished: bool,
    steps_taken: u32,
    last_action: usize,
    // **NEW**: Track action history for Q-learning updates
    action_history: Vec<(String, usize, i32, i32)>, // (state_hash, action, x, y)
}
```

**Action Recording**:
- Each car action is recorded with: `(state_hash, action, x, y)`
- State hash is generated based on current position and surrounding tiles
- Actions are recorded before tile effects are applied

### 2. Reward Calculation

**Base Reward Types**:
- **Rank Rewards**: Based on finishing position (1st = 100, 2nd = 50, etc.)
- **Distance Rewards**: Based on progress made during the race
- **Stuck Penalties**: For cars that get stuck during the race

**Action-Specific Adjustments**:
- **Forward Movement** (`ACTION_RIGHT`): +10 bonus
- **Backward Movement** (`ACTION_LEFT`): -5 penalty
- **Lateral Movement** (`ACTION_UP/DOWN`): Neutral
- **Staying** (`ACTION_STAY`): -10 penalty

### 3. Q-Update Generation

**Location**: `contracts/race-engine/src/contract.rs`

**QUpdate Structure**:
```rust
struct QUpdate {
    car_id: String,
    state_hash: String,
    action: u8,
    reward_type: RewardType,
    next_state_hash: Option<String>,
}
```

**Generation Process**:
1. For each car, calculate base reward based on race performance
2. For each action in car's history, calculate action-specific reward
3. Create QUpdate with current state, action, reward, and next state
4. Collect all QUpdates into a vector

### 4. Trainer Contract Integration

**Race Engine → Trainer Communication**:
```rust
// Calculate Q-updates based on race results
let q_updates = calculate_q_updates(&race_state, &race_result)?;

// Get trainer contract address from config
let config = get_config(deps.storage)?;
let trainer_contract = config.trainer_contract;

// Create message to update trainer contract
let trainer_msg = racing::trainer::ExecuteMsg::BatchUpdateQValues { updates: q_updates };
let wasm_msg = WasmMsg::Execute {
    contract_addr: trainer_contract,
    msg: to_json_binary(&trainer_msg)?,
    funds: vec![],
};
```

## Reward Calculation Details

### Race Performance Rewards

**Finished Cars**:
- **Winners**: `RewardType::Rank(0)` → 100 points
- **2nd Place**: `RewardType::Rank(1)` → 50 points
- **3rd Place**: `RewardType::Rank(2)` → 25 points
- **4th Place**: `RewardType::Rank(3)` → 10 points

**Unfinished Cars**:
- **Stuck Cars**: `RewardType::Stuck` → -5 points
- **Progress Cars**: `RewardType::Distance(progress)` → Based on steps taken

### Action-Specific Rewards

**Forward Movement Bonus**:
- Encourages cars to move toward the finish line
- `progress + 10` for rightward movement

**Backward Movement Penalty**:
- Discourages cars from moving away from finish
- `progress - 5` for leftward movement

**Staying Penalty**:
- Encourages continuous movement
- `progress - 10` for staying in place

## State Hash Generation

**Location**: `contracts/race-engine/src/contract.rs`

**State Hash Components**:
1. **Current Tile Type**: Normal, Wall, Sticky, Boost, Slow, Finish
2. **Distance from Finish**: Numeric distance value
3. **Surrounding Tiles**: 8 surrounding tile types for context

**Example State Hash**:
```
"Normal,5,Wall,Normal,Sticky,Normal,Wall,Normal,Wall"
```

## Configuration

**Race Engine Config**:
```rust
struct Config {
    admin: String,
    max_ticks: u32,
    max_recent_races: u32,
    trainer_contract: String, // **NEW**: Trainer contract address
}
```

**Instantiation**:
```rust
InstantiateMsg {
    admin: "admin_address",
    trainer_contract: "trainer_contract_address",
}
```

## Benefits

### 1. **Automatic Learning**
- No manual intervention required
- Cars learn from every race automatically
- Continuous improvement over time

### 2. **Realistic Rewards**
- Rewards based on actual race performance
- Action-specific adjustments for better learning
- Penalties for poor behavior (stuck, backward movement)

### 3. **Scalable Architecture**
- Batch updates reduce gas costs
- Configurable trainer contract address
- Modular reward calculation system

### 4. **Complete Feedback Loop**
- Race results directly influence learning
- Actions are tied to specific states
- Next-state information enables proper Q-learning

## Usage Example

```rust
// Simulate a race
let response = execute_simulate_race(
    deps,
    env,
    "track_1".to_string(),
    vec!["car_1".to_string(), "car_2".to_string(), "car_3".to_string()]
)?;

// This automatically:
// 1. Simulates the race
// 2. Records all car actions
// 3. Calculates rewards based on performance
// 4. Sends batch Q-updates to trainer contract
// 5. Updates Q-values for all cars
```

## Error Handling

**Race Engine Errors**:
- `InvalidCarCount`: Too few or too many cars
- `SimulationError`: Race simulation fails
- `QLearningError`: Q-update calculation fails

**Trainer Contract Errors**:
- `InvalidAction`: Invalid action index
- `Storage`: Storage operation fails

## Future Enhancements

### 1. **Advanced Reward Shaping**
- Track-specific reward adjustments
- Time-based rewards
- Collision penalties

### 2. **Multi-Race Learning**
- Aggregate rewards across multiple races
- Long-term performance tracking
- Adaptive reward scaling

### 3. **Real-time Updates**
- Immediate Q-value updates during race
- Progressive learning within single race
- Dynamic reward adjustment

### 4. **Performance Analytics**
- Learning rate tracking
- Convergence metrics
- Performance comparison tools 