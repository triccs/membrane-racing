# Race Simulation System Documentation

## Overview

The race simulation system is a deterministic, turn-based racing game where AI-controlled cars navigate through 2D grid-based tracks using Q-learning. The system supports both training and competitive racing modes.

## Core Components

### 1. Data Structures

#### CarState
```rust
struct CarState {
    car_id: String,                    // Unique car identifier
    tile: TrackTile,                   // Current tile the car is on
    x: i32, y: i32,                   // Current position
    stuck: bool,                       // Whether car is stuck (skip next turn)
    finished: bool,                    // Whether car has finished the race
    steps_taken: u32,                  // Total steps taken
    last_action: usize,                // Last action taken (0-4)
    action_history: Vec<(String, usize, TrackTile)>, // For Q-learning
    hit_wall: bool,                    // Whether car hit a wall this turn
    current_speed: u32,                // Current speed modifier
}
```

#### RaceState
```rust
struct RaceState {
    cars: Vec<CarState>,              // All cars in the race
    track_layout: Vec<Vec<TrackTile>>, // 2D track layout
    tick: u32,                        // Current tick number
    play_by_play: Vec<String>,        // Race log
}
```

#### TileProperties
```rust
struct TileProperties {
    speed_modifier: u32,              // Speed multiplier (2=normal, 1=slow, 3=boost)
    blocks_movement: bool,            // Whether tile blocks movement
    skip_next_turn: bool,             // Whether tile causes skip next turn
    damage: i32,                      // Damage/healing amount
    is_finish: bool,                  // Whether tile is finish line
    is_start: bool,                   // Whether tile is start line
}
```

### 2. Action System

#### Available Actions (5 total)
- `ACTION_UP (0)`: Move up
- `ACTION_DOWN (1)`: Move down  
- `ACTION_LEFT (2)`: Move left
- `ACTION_RIGHT (3)`: Move right
- `ACTION_STAY (4)`: Stay in place

#### Action Selection Strategies
- **Best**: Always choose highest Q-value action
- **Random**: Pure random selection
- **EpsilonGreedy(ε)**: ε% random, (1-ε)% best
- **Softmax(T)**: Probabilistic based on Q-values with temperature T

### 3. Tile System

#### Tile Types and Effects
- **Normal**: `speed_modifier: 2` - Standard movement
- **Boost**: `speed_modifier > 2` - Increased speed
- **Slow**: `speed_modifier < 2` - Reduced speed
- **Wall**: `blocks_movement: true` - Blocks movement
- **Sticky**: `skip_next_turn: true` - Move but skip next turn
- **Finish**: `is_finish: true` - Race completion
- **Start**: `is_start: true` - Starting position
- **Damage**: `damage > 0` - Deals damage
- **Healing**: `damage < 0` - Restores health

## Race Simulation Flow

### 1. Race Initialization
```rust
fn execute_simulate_race(
    track_id: String,
    car_ids: Vec<String>,
    training_config: TrainingConfig,
    reward_config: Option<RewardNumbers>
) -> Result<Response, ContractError>
```

**Steps:**
1. Validate input (1-8 cars)
2. Load track from track manager
3. Find start positions
4. Initialize car states with default values
5. Create race state
6. Run simulation
7. Calculate results and Q-updates
8. Send updates to trainer contract

### 2. Main Simulation Loop
```rust
fn simulate_race(race_state: &mut RaceState, training_config: TrainingConfig) -> Result<RaceResult, ContractError>
```

**Loop Conditions:**
- Continue while `tick < MAX_TICKS` (100)
- Continue while not all cars finished

**Per Tick:**
1. Record tick start in play-by-play
2. Simulate one tick
3. Increment tick counter

### 3. Tick Simulation
```rust
fn simulate_tick(race_state: &mut RaceState, training_config: TrainingConfig, tick_index: u32)
```

**Phase 1: Reset States**
- Reset car states for new tick
- Clear `hit_wall` flags
- Reset `stuck` status

**Phase 2: Calculate Intended Moves**
For each car:
1. Skip if finished or stuck
2. Determine action strategy based on training config
3. Get action from Q-table or heuristic
4. Use car's current speed for movement
5. Calculate new position
6. Check for wall collisions

**Phase 3: Collision Resolution**
- Check for car-to-car collisions
- If collision detected, cars stay in place
- Otherwise, use intended positions

**Phase 4: Apply Tile Effects**
For each car:
1. Record action in history for Q-learning
2. Apply tile effects using properties
3. Update car position and state

**Phase 5: Logging**
- Record final positions in play-by-play

### 4. Tile Effect Application
```rust
fn apply_tile_effects_to_car(car: &mut CarState, new_x: i32, new_y: i32, track_layout: &[Vec<TrackTile>])
```

**Speed Modifiers:**
- Set `car.current_speed = tile.speed_modifier`
- Applies immediately for all tiles

**Special Effects:**
- **Finish**: Set `finished = true`, move to tile
- **Start**: Normal movement
- **Wall**: Stay in place, increment steps
- **Sticky**: Move to tile, set `stuck = true`
- **Normal**: Move to tile, increment steps

### 5. Position Calculation
```rust
fn calculate_new_position(x: i32, y: i32, action: usize, tiles_moved: u32, track_layout: &[Vec<TrackTile>])
```

**Movement Logic:**
1. Calculate direction based on action
2. Move `tiles_moved` tiles in that direction
3. Check if target tile blocks movement
4. If blocked, bounce back and set `hit_wall = true`
5. Return new position and wall collision status

### 6. Action Selection
```rust
fn get_car_action(car_id: String, track_layout: &[Vec<TrackTile>], x: i32, y: i32, strategy: ActionSelectionStrategy, seed: u32)
```

**Q-Table Query:**
- Generate state hash based on position and surroundings
- Query Q-table for car (currently returns placeholder values)
- Apply strategy to select action

**State Hash Generation:**
```rust
fn generate_state_hash(track_layout: &[Vec<TrackTile>], x: i32, y: i32) -> String
```
Includes:
- Current tile properties (speed, blocks, sticky, damage, finish, start)
- Progress towards finish
- Surrounding tile properties

### 7. Result Calculation
```rust
fn calculate_results(cars: &[CarState]) -> (Vec<String>, Vec<(String, u32)>, Vec<(String, u32)>)
```

**Winner Determination:**
1. Separate finished and unfinished cars
2. Sort finished cars by steps taken (lower = better)
3. Sort unfinished cars by position heuristic
4. Winners are finished cars with lowest steps

**Rankings:**
- Finished cars ranked by steps taken
- Unfinished cars ranked by position

## Q-Learning Integration

### 1. Action History Tracking
Each car maintains an action history:
```rust
action_history: Vec<(String, usize, TrackTile)>
// (state_hash, action, tile)
```

### 2. Q-Update Calculation
```rust
fn calculate_q_updates(race_state: &RaceState, race_result: &RaceResult, reward_config: RewardNumbers)
```

**Process:**
1. For each car, process action history
2. Calculate reward for each action
3. Create Q-update with state hash, action, reward, next state
4. Send batch updates to trainer contract

### 3. Reward Calculation
```rust
fn calculate_action_reward(car: &CarState, race_result: &RaceResult, action: usize, last_tile: TrackTile, tile: TrackTile, ...)
```

**Reward Components:**
- **Rank rewards**: Based on final position (1st=100, 2nd=50, etc.)
- **Wall penalty**: -8 for hitting walls
- **Stuck penalty**: -5 for getting stuck
- **Distance reward**: Based on progress towards finish
- **No-move penalty**: 0 for staying in place

## Key Features

### 1. Deterministic Simulation
- Uses pseudo-random number generator with seed
- Same inputs always produce same results
- Critical for on-chain verification

### 2. Property-Based Tile System
- No hardcoded tile types
- All effects determined by tile properties
- Highly extensible and flexible

### 3. Speed Modifier System
- Cars have individual speed states
- Speed affects movement distance per turn
- Immediate application of speed changes

### 4. Collision Detection
- Car-to-car collision prevention
- Cars stay in place if collision detected
- Simple but effective collision resolution

### 5. Comprehensive Logging
- Play-by-play recording of each tick
- Car positions and actions logged
- Useful for debugging and replay

## Limitations and Issues

### 1. Q-Table Implementation
- Currently returns placeholder values
- Needs integration with actual car Q-tables
- `query_car_q_table()` function incomplete

### 2. Damage System
- Damage/healing properties exist but not implemented
- TODO comment indicates future implementation needed

### 3. Distance Calculation
- Uses simple position-based heuristic for unfinished cars
- Could be improved with actual pathfinding distance

### 4. State Hash Compression
- Current state hash includes all surrounding tiles
- Could be optimized for Q-table efficiency
- Mentioned in comments as potential improvement

## Performance Characteristics

### Time Complexity
- **Per tick**: O(n²) where n = number of cars (collision detection)
- **Per race**: O(t × n) where t = ticks, n = cars
- **Q-updates**: O(n × a) where a = average actions per car

### Space Complexity
- **Race state**: O(w × h + n) where w×h = track size, n = cars
- **Action history**: O(n × t) for all cars' action histories
- **Play-by-play**: O(t) for tick-by-tick logging

## Future Improvements

1. **Abstract Actions**: Simplify Q-table by using more abstract actions
2. **State Hash Compression**: Optimize state representation without losing information
3. **Damage System**: Implement health/damage mechanics
4. **Pathfinding**: Improve distance calculations for unfinished cars
5. **Q-Table Integration**: Complete the car Q-table query system

## Conclusion

The race simulation system provides a robust, deterministic foundation for AI racing with Q-learning. The property-based tile system offers great flexibility, while the comprehensive logging and Q-learning integration support both training and competitive racing scenarios. The main areas for improvement are completing the Q-table integration and implementing the damage system. 