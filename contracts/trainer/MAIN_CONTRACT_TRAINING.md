# Main Contract Training Methods

## 🎯 **Overview**

The main trainer contract now includes comprehensive anti-stuck training methods that implement the strategies we developed in tests. All training logic is now in the main contract, not just in tests.

## 🚀 **New Training Methods**

### **1. Enhanced Training (`EnhancedTraining`)**
```rust
ExecuteMsg::EnhancedTraining {
    car_id: String,
    training_rounds: u32,
    steps_per_round: u32,
    exploration_rate: f32,
    reward_multiplier: f32,
}
```

**Features:**
- Enhanced reward structure with anti-stuck incentives
- Configurable exploration rate and reward multiplier
- Anti-stuck action selection with enhanced exploration
- Tracks optimal actions and stuck actions

**Strategy:**
- Early steps: prefer forward movement with some exploration
- Later steps: more exploration to prevent stuck
- Enhanced rewards for forward movement
- Configurable exploration rate

### **2. Progressive Learning (`ProgressiveLearning`)**
```rust
ExecuteMsg::ProgressiveLearning {
    car_id: String,
    initial_exploration: f32,
    final_exploration: f32,
    learning_rounds: u32,
    steps_per_round: u32,
}
```

**Features:**
- Exploration rate that decreases over time
- Progressive reward structure
- Controlled exploration to prevent stuck
- Tracks learning progression

**Strategy:**
- Starts with high exploration (initial_exploration)
- Gradually reduces to final_exploration
- Early steps: mostly forward with limited exploration
- Later steps: more exploration to prevent stuck

### **3. Anti-Stuck Training (`AntiStuckTraining`)**
```rust
ExecuteMsg::AntiStuckTraining {
    car_id: String,
    track_length: u32,
    training_rounds: u32,
    stuck_threshold: u32,
    force_forward_after: u32,
}
```

**Features:**
- Specific stuck prevention logic
- Configurable stuck threshold and force forward parameters
- Tracks consecutive non-forward actions
- Forces forward movement when stuck

**Strategy:**
- Monitors consecutive non-forward actions
- Forces forward movement if stuck threshold reached
- Enhanced reward structure for anti-stuck
- Tracks forced forward moves

### **4. Smart Training (`SmartTraining`)**
```rust
ExecuteMsg::SmartTraining {
    car_id: String,
    track_config: TrackTrainingConfig,
    training_strategy: TrainingStrategy,
}
```

**Features:**
- Combines multiple strategies
- Configurable track parameters
- Multiple training strategies
- Smart action selection

**Strategy Options:**
- `Random`: High exploration (80%)
- `Guided`: Balanced exploration (40%)
- `AntiStuck`: Anti-stuck focused (60%)
- `Progressive`: Decreasing exploration
- `Enhanced`: Low exploration, high rewards (30%)

## 📊 **Track Training Configuration**

### **TrackTrainingConfig**
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

**Example Configuration:**
```rust
TrackTrainingConfig {
    track_length: 20,
    early_stage_end: 5,
    middle_stage_end: 10,
    late_stage_end: 15,
    early_reward: 25,
    middle_reward: 40,
    late_reward: 60,
    finish_reward: 100,
}
```

## 🔍 **Query Methods**

### **1. Training Progress Query**
```rust
QueryMsg::GetTrainingProgress { car_id: String }
```

**Response:**
```rust
pub struct GetTrainingProgressResponse {
    pub car_id: String,
    pub total_rounds: u32,
    pub current_round: u32,
    pub optimal_actions: u32,
    pub stuck_actions: u32,
    pub learning_efficiency: f32,
    pub stuck_prevention_rate: f32,
}
```

### **2. Anti-Stuck Metrics Query**
```rust
QueryMsg::GetAntiStuckMetrics { car_id: String }
```

**Response:**
```rust
pub struct GetAntiStuckMetricsResponse {
    pub car_id: String,
    pub avg_optimal_actions: f32,
    pub avg_stuck_actions: f32,
    pub learning_efficiency: f32,
    pub stuck_prevention_rate: f32,
    pub track_completion_rate: f32,
    pub q_value_diversity: f32,
}
```

## 🎯 **Usage Examples**

### **Example 1: Enhanced Training**
```rust
let msg = ExecuteMsg::EnhancedTraining {
    car_id: "my_car".to_string(),
    training_rounds: 30,
    steps_per_round: 20,
    exploration_rate: 0.3,
    reward_multiplier: 1.5,
};
```

### **Example 2: Progressive Learning**
```rust
let msg = ExecuteMsg::ProgressiveLearning {
    car_id: "my_car".to_string(),
    initial_exploration: 0.5,
    final_exploration: 0.1,
    learning_rounds: 25,
    steps_per_round: 20,
};
```

### **Example 3: Anti-Stuck Training**
```rust
let msg = ExecuteMsg::AntiStuckTraining {
    car_id: "my_car".to_string(),
    track_length: 20,
    training_rounds: 40,
    stuck_threshold: 2,
    force_forward_after: 3,
};
```

### **Example 4: Smart Training**
```rust
let track_config = TrackTrainingConfig {
    track_length: 20,
    early_stage_end: 5,
    middle_stage_end: 10,
    late_stage_end: 15,
    early_reward: 25,
    middle_reward: 40,
    late_reward: 60,
    finish_reward: 100,
};

let msg = ExecuteMsg::SmartTraining {
    car_id: "my_car".to_string(),
    track_config,
    training_strategy: TrainingStrategy::AntiStuck,
};
```

## 🔧 **Key Features**

### **1. Anti-Stuck Logic**
- Detects when car is stuck (repeated non-forward actions)
- Forces forward movement after stuck detection
- Configurable stuck thresholds
- Tracks forced forward moves

### **2. Enhanced Rewards**
- Higher reward values for forward movement
- Progressive reward structure
- Configurable reward multipliers
- Clear progression in reward values

### **3. Progressive Learning**
- Exploration rate that decreases over time
- Starts with high exploration, ends with low exploration
- Balances exploration and exploitation
- Maintains some exploration to prevent stuck

### **4. Smart Action Selection**
- Strategy-based action selection
- Configurable exploration rates
- Anti-stuck action diversity
- Forward movement bias

### **5. Comprehensive Metrics**
- Tracks optimal actions vs stuck actions
- Learning efficiency calculation
- Stuck prevention rate
- Q-value diversity analysis

## 📈 **Training Constants**

```rust
// Anti-stuck training constants
const DEFAULT_EXPLORATION_RATE: f32 = 0.3;
const DEFAULT_REWARD_MULTIPLIER: f32 = 1.5;
const DEFAULT_STUCK_THRESHOLD: u32 = 2;
const DEFAULT_FORCE_FORWARD_AFTER: u32 = 3;
```

## 🎯 **Success Criteria**

### **Learning Efficiency**
- Target: ≥ 80% optimal actions
- Measured: optimal_actions / total_updates

### **Stuck Prevention**
- Target: ≤ 20% stuck actions
- Measured: stuck_actions / total_updates

### **Track Completion**
- Target: ≥ 90% completion rate
- Measured: final_position / track_length

## 🚀 **Benefits**

### **1. Main Contract Implementation**
- Training logic is now in the actual contract
- No longer just test-only functionality
- Can be called by other contracts
- Persistent training state

### **2. Configurable Strategies**
- Multiple training strategies available
- Configurable parameters
- Track-specific configurations
- Strategy-specific optimizations

### **3. Comprehensive Monitoring**
- Real-time training progress
- Anti-stuck metrics
- Learning efficiency tracking
- Q-value analysis

### **4. Anti-Stuck Prevention**
- Proactive stuck detection
- Forced forward movement
- Enhanced exploration
- Progressive learning

## 🎯 **Conclusion**

The main contract now includes comprehensive anti-stuck training methods that:

1. **✅ Implement All Strategies**: Enhanced, Progressive, Anti-Stuck, and Smart training
2. **✅ Prevent Stuck Behavior**: Multiple anti-stuck mechanisms
3. **✅ Provide Monitoring**: Real-time metrics and progress tracking
4. **✅ Enable Configuration**: Flexible parameters and strategies
5. **✅ Support Integration**: Can be called by other contracts

All training logic is now properly implemented in the main contract, not just in tests. The car can be trained using these methods to prevent getting stuck and achieve optimal track completion! 