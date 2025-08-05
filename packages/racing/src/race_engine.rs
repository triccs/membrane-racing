use std::collections::HashMap;

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

use crate::types::{QTableEntry, RewardNumbers, Track, TrackTile};

pub const DEFAULT_SPEED: u8 = 1;
pub const DEFAULT_BOOST_SPEED: u8 = 3;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
    pub track_contract: String,
    pub car_contract: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    SimulateRace {
        track_id: Uint128,
        car_ids: Vec<String>,
        train: bool,
        training_config: Option<TrainingConfig>,
        reward_config: Option<RewardNumbers>,
    },
    /// Reset the Q-table for a car
    /// Must be called by the owner of the car in the car contract
    ResetQ {
        car_id: Uint128,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(RaceResultResponse)]
    GetRaceResult { 
        track_id: String,
        race_id: String,
     },
    #[returns(RecentRacesResponse)]
    ListRecentRaces {
        ///Must provide one of the following////
        //Filter by car id 
        /// - If provided, return races for that car
        car_id: Option<String>,
        //Filter by track id
        /// - If provided, return races for that track
        track_id: Option<String>,
        //Start after a specific race id
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(ConfigResponse)]
    GetConfig {},
    #[returns(GetQResponse)]
    GetQ { car_id: String, state_hash: Option< [u8; 32]> },
}

#[cw_serde]
pub struct RaceResultResponse {
    pub result: RaceResult,
}
#[cw_serde]
pub struct GetQResponse {
    pub car_id: String,
    pub q_values: Vec<QTableEntry>,
}

#[cw_serde]
pub struct RecentRacesResponse {
    pub races: Vec<RaceResult>,
}

#[cw_serde]
pub struct ConfigResponse {
    pub config: Config,
}

#[cw_serde]
pub struct Rank {
    pub car_id: String,
    pub rank: u32,
}


#[cw_serde]
pub struct Position {
    pub car_id: String,
    pub x: u32,
    pub y: u32,
}

#[cw_serde]
pub struct Action {
    pub action: String,
    pub resulting_position: Position,
}

#[cw_serde]
pub struct PlayByPlay {
    pub starting_position: Position,
    pub actions: Vec<Action>,
}

#[cw_serde]
pub struct Step {
    pub car_id: String,
    pub steps_taken: u32,
}

#[cw_serde]
pub struct RaceResult {
    pub race_id: String,
    pub track_id: Uint128,
    pub car_ids: Vec<String>,
    pub winner_ids: Vec<String>,
    pub rankings: Vec<Rank>,
    pub play_by_play: HashMap<String, PlayByPlay>,
    pub steps_taken: Vec<Step>,
}



#[cw_serde]
pub struct CarState {
    pub car_id: String,
    pub tile: TrackTile,
    pub x: i32,
    pub y: i32,
    pub stuck: bool,
    pub finished: bool,
    pub steps_taken: u32,
    pub last_action: usize,
    // **NEW**: Track action history for Q-learning updates
    pub action_history: Vec<( [u8; 32], usize, TrackTile)>, // (state_hash, action, tile)
    // **NEW**: Track wall collisions for reward calculation
    pub hit_wall: bool,
    // **NEW**: Track speed modifiers
    pub current_speed: u32,
    // **NEW**: Store used Q-table for this car
    pub q_table:  Vec<QTableEntry>, 
}

#[cw_serde]
pub struct RaceState {
    pub cars: Vec<CarState>,
    pub track_layout: Vec<Vec<TrackTile>>,
    pub tick: u32,
    pub play_by_play: std::collections::HashMap<String, PlayByPlay>,
}


#[cw_serde]
pub struct Config {
    pub admin: String,
    pub track_contract: String,
    pub car_contract: String,
    pub max_ticks: u32,
    pub max_recent_races: u32,
} 

#[cw_serde]
pub struct TrainingConfig {
    pub training_mode: bool,
    pub epsilon: f32,
    pub temperature: f32,
    pub enable_epsilon_decay: bool,
}