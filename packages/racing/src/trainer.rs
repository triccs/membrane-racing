use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::types::{RewardType, QUpdate};

#[cw_serde]
pub struct InstantiateMsg {
    /// The admin address that can control the trainer contract
    pub admin: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Update multiple Q-values in a batch operation
    BatchUpdateQValues {
        /// List of Q-value updates to process
        updates: Vec<QUpdate>,
    },
    /// Start a training session with configurable parameters
    StartTraining {
        /// Training request with configuration
        training_request: TrainingRequest,
    },
}

/// Training request with configurable parameters
#[cw_serde]
pub struct TrainingRequest {
    /// Unique identifier for the car being trained
    pub car_id: String,
    /// Unique identifier for the track (optional, uses default if not specified)
    pub track_id: Option<String>,
    /// Training configuration (optional, falls back to contract constants)
    pub training_config: Option<TrainingConfig>,
    /// Reward configuration (optional, falls back to contract constants)
    pub reward_config: Option<RewardConfig>,
    /// Type of training strategy to use
    pub training_type: TrainingType,
}

/// Training configuration parameters
#[cw_serde]
pub struct TrainingConfig {
    /// Number of training rounds to perform
    pub training_rounds: u32,
    /// Number of steps per training round
    pub steps_per_round: u32,
    /// Rate of exploration vs exploitation (0.0-1.0)
    pub exploration_rate: f32,
    /// Multiplier for reward values
    pub reward_multiplier: f32,
    /// Number of consecutive non-forward actions before forcing forward
    pub stuck_threshold: u32,
    /// Number of consecutive forward actions before allowing exploration
    pub force_forward_after: u32,
    /// Whether to use progressive exploration (decreasing over time)
    pub use_progressive_exploration: bool,
    /// Initial exploration rate for progressive learning (if enabled)
    pub initial_exploration: Option<f32>,
    /// Final exploration rate for progressive learning (if enabled)
    pub final_exploration: Option<f32>,
    /// Whether to enable anti-stuck mechanisms
    pub enable_anti_stuck: bool,
    /// Whether to track detailed metrics
    pub track_metrics: bool,
}

/// Reward configuration parameters
#[cw_serde]
pub enum RewardConfig {
    /// Use a predefined reward template
    Template(RewardTemplate),
    /// Use custom reward parameters
    Custom(CustomRewardParams),
}

/// Training type enum
#[cw_serde]
pub enum TrainingType {
    /// Anti-stuck focused training
    AntiStuck,
    /// Progressive learning with decreasing exploration
    Progressive,
    /// Enhanced training with high rewards
    Enhanced,
    /// Balanced training approach
    Balanced,
    /// Custom training with user-defined parameters
    Custom,
}

/// Reward templates with specific properties
#[cw_serde]
pub enum RewardTemplate {
    /// Anti-stuck focused reward template
    AntiStuck {
        /// Base reward for forward movement
        forward_reward: i32,
        /// Penalty for stuck behavior
        stuck_penalty: i32,
        /// Bonus for exploration
        exploration_bonus: i32,
        /// Multiplier for anti-stuck rewards
        anti_stuck_multiplier: f32,
    },
    /// Speed-focused reward template
    Speed {
        /// Base reward for forward movement
        forward_reward: i32,
        /// Bonus for fast completion
        speed_bonus: i32,
        /// Penalty for slow movement
        slow_penalty: i32,
        /// Multiplier for speed rewards
        speed_multiplier: f32,
    },
    /// Conservative reward template
    Conservative {
        /// Base reward for forward movement
        forward_reward: i32,
        /// Penalty for risky actions
        risk_penalty: i32,
        /// Bonus for safe movement
        safety_bonus: i32,
        /// Multiplier for conservative rewards
        conservative_multiplier: f32,
    },
    /// Aggressive reward template
    Aggressive {
        /// Base reward for forward movement
        forward_reward: i32,
        /// Bonus for aggressive actions
        aggressive_bonus: i32,
        /// Penalty for conservative actions
        conservative_penalty: i32,
        /// Multiplier for aggressive rewards
        aggressive_multiplier: f32,
    },
    /// Balanced reward template
    Balanced {
        /// Base reward for forward movement
        forward_reward: i32,
        /// Balanced exploration bonus
        exploration_bonus: i32,
        /// Balanced penalty for stuck behavior
        stuck_penalty: i32,
        /// Multiplier for balanced rewards
        balanced_multiplier: f32,
    },
}

/// Custom reward parameters for user-defined reward functions
#[cw_serde]
pub struct CustomRewardParams {
    /// Base reward for forward movement
    pub forward_reward: i32,
    /// Reward for upward movement
    pub up_reward: i32,
    /// Reward for downward movement
    pub down_reward: i32,
    /// Reward for leftward movement
    pub left_reward: i32,
    /// Reward for staying in place
    pub stay_reward: i32,
    /// Penalty for stuck behavior
    pub stuck_penalty: i32,
    /// Penalty for wall collision
    pub wall_penalty: i32,
    /// Penalty for no movement
    pub no_move_penalty: i32,
    /// Bonus for exploration
    pub exploration_bonus: i32,
    /// Multiplier for all rewards
    pub reward_multiplier: f32,
    /// Stage-specific rewards
    pub stage_rewards: StageRewards,
    /// Rank rewards for tournament performance
    pub rank_rewards: [i32; 4], // 1st, 2nd, 3rd, 4th place
}

/// Stage-specific rewards for track progression
#[cw_serde]
pub struct StageRewards {
    /// Reward for early stage progress (0-5 positions)
    pub early_stage_reward: i32,
    /// Reward for middle stage progress (6-10 positions)
    pub middle_stage_reward: i32,
    /// Reward for late stage progress (11-15 positions)
    pub late_stage_reward: i32,
    /// Reward for finishing the track
    pub finish_reward: i32,
}

/// Active training session stored in state
#[cw_serde]
pub struct TrainingSession {
    /// Car ID being trained
    pub car_id: String,
    /// Track ID being used for training
    pub track_id: String,
    /// Training configuration
    pub training_config: TrainingConfig,
    /// Reward configuration
    pub reward_config: RewardConfig,
    /// Training type
    pub training_type: TrainingType,
    /// Current training round
    pub current_round: u32,
    /// Total rounds completed
    pub total_rounds: u32,
    /// Training start timestamp
    pub start_time: u64,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Get training statistics for a specific car
    #[returns(GetCarTrainingStatsResponse)]
    GetCarTrainingStats { 
        /// Unique identifier for the car
        car_id: String 
    },
    /// Get track results for a specific car on a specific track
    #[returns(GetTrackResultsResponse)]
    GetTrackResults { 
        /// Unique identifier for the car
        car_id: String, 
        /// Unique identifier for the track
        track_id: String 
    },
    /// Get Q-values for a specific car in a specific state
    #[returns(GetQValueResponse)]
    GetQValue { 
        /// Unique identifier for the car
        car_id: String, 
        /// Hash representing the state
        state_hash: String 
    },
    /// Get training progress metrics for a specific car
    #[returns(GetTrainingProgressResponse)]
    GetTrainingProgress { 
        /// Unique identifier for the car
        car_id: String 
    },
    /// Get anti-stuck metrics for a specific car
    #[returns(GetAntiStuckMetricsResponse)]
    GetAntiStuckMetrics { 
        /// Unique identifier for the car
        car_id: String 
    },
    /// Get current training session for a car
    #[returns(GetTrainingSessionResponse)]
    GetTrainingSession { 
        /// Unique identifier for the car
        car_id: String 
    },
    /// Get available reward templates
    #[returns(GetRewardTemplatesResponse)]
    GetRewardTemplates {},
}

#[cw_serde]
pub struct GetCarTrainingStatsResponse {
    /// Unique identifier for the car
    pub car_id: String,
    /// Total number of training updates performed
    pub training_updates: u32,
}

#[cw_serde]
pub struct GetTrackResultsResponse {
    /// Unique identifier for the car
    pub car_id: String,
    /// Unique identifier for the track
    pub track_id: String,
    /// Number of wins on this track
    pub wins: u32,
    /// Number of losses on this track
    pub losses: u32,
}

#[cw_serde]
pub struct GetQValueResponse {
    /// Unique identifier for the car
    pub car_id: String,
    /// Hash representing the state
    pub state_hash: String,
    /// Q-values for all 5 actions [Forward, Up, Down, Left, Stay]
    pub q_values: [i32; 5],
}

#[cw_serde]
pub struct GetTrainingProgressResponse {
    /// Unique identifier for the car
    pub car_id: String,
    /// Total number of training rounds completed
    pub total_rounds: u32,
    /// Current training round number
    pub current_round: u32,
    /// Number of optimal actions (forward movements) performed
    pub optimal_actions: u32,
    /// Number of stuck actions (non-forward movements) performed
    pub stuck_actions: u32,
    /// Learning efficiency as percentage (0.0-100.0)
    pub learning_efficiency: f32,
    /// Stuck prevention rate as percentage (0.0-100.0)
    pub stuck_prevention_rate: f32,
}

#[cw_serde]
pub struct GetAntiStuckMetricsResponse {
    /// Unique identifier for the car
    pub car_id: String,
    /// Average number of optimal actions per round
    pub avg_optimal_actions: f32,
    /// Average number of stuck actions per round
    pub avg_stuck_actions: f32,
    /// Overall learning efficiency as percentage (0.0-100.0)
    pub learning_efficiency: f32,
    /// Overall stuck prevention rate as percentage (0.0-100.0)
    pub stuck_prevention_rate: f32,
    /// Track completion rate as percentage (0.0-100.0)
    pub track_completion_rate: f32,
    /// Diversity of Q-values (number of different Q-values > 0)
    pub q_value_diversity: f32,
}

/// Response for current training session
#[cw_serde]
pub struct GetTrainingSessionResponse {
    /// Current training session (None if no active session)
    pub session: Option<TrainingSession>,
}

/// Response for available reward templates
#[cw_serde]
pub struct GetRewardTemplatesResponse {
    /// List of available reward templates
    pub templates: Vec<RewardTemplateInfo>,
}

/// Information about a reward template
#[cw_serde]
pub struct RewardTemplateInfo {
    /// Name of the template
    pub name: String,
    /// Description of the template
    pub description: String,
    /// Template type
    pub template_type: String,
    /// Default parameters for this template
    pub default_params: RewardTemplate,
    /// Recommended use cases
    pub recommended_use: Vec<String>,
} 