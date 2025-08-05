use cosmwasm_std::{StdError, StdResult, Storage};
use cw_storage_plus::{Item, Map};
use serde::{Deserialize, Serialize};

use racing::race_engine::{Config, RaceResult};

pub const CONFIG: Item<Config> = Item::new("config");
pub const CAR_RECENT_RACES: Map<&str, Vec<RaceResult>> = Map::new("car_recent_races");
pub const TRACK_RECENT_RACES: Map<&str, Vec<RaceResult>> = Map::new("track_recent_races");

// Constants
pub const MAX_CAR_RECENT_RACES: usize = 9;
pub const MAX_TRACK_RECENT_RACES: usize = 32;
pub const MAX_TICKS: u32 = 100;


// Q-table storage: (car_id, state_hash) -> [i32; 4] action values
pub const Q_TABLE: Map<(&str, &[u8; 32]), [i32; 4]> = Map::new("q_table");

pub fn get_q_values(storage: &dyn Storage, car_id: &str, state_hash: & [u8; 32]) -> StdResult<[i32; 4]> {
    Q_TABLE.load(storage, (car_id, state_hash))
}

pub fn set_q_values(
    storage: &mut dyn Storage,
    car_id: &str,
    state_hash: &[u8; 32],
    q_values: [i32; 4],
) -> StdResult<()> {
    Q_TABLE.save(storage, (car_id, state_hash), &q_values)
}


pub fn get_config(storage: &dyn cosmwasm_std::Storage) -> StdResult<Config> {
    CONFIG.load(storage)
}

pub fn set_config(storage: &mut dyn cosmwasm_std::Storage, config: Config) -> StdResult<()> {
    CONFIG.save(storage, &config)
}

pub fn get_recent_races(storage: &dyn cosmwasm_std::Storage, car_id: Option<String>, track_id: Option<String>) -> StdResult<Vec<RaceResult>> {
    if let Some(car_id) = car_id {
        CAR_RECENT_RACES.load(storage, &car_id)
    } else if let Some(track_id) = track_id {
        TRACK_RECENT_RACES.load(storage, &track_id)
    } else {
        return Err(StdError::generic_err("No car or track ID provided"));
    }
}

pub fn add_recent_race(storage: &mut dyn cosmwasm_std::Storage, race_result: RaceResult, car_id: Option<String>, track_id: Option<String>) -> StdResult<()> {
    let mut races = if let Some(car_id) = car_id.clone() {
        CAR_RECENT_RACES.load(storage, &car_id).unwrap_or_default()
    } else if let Some(track_id) = track_id.clone() {
        TRACK_RECENT_RACES.load(storage, &track_id).unwrap_or_default()
    } else {
        return Err(StdError::generic_err("No car or track ID provided"));
    };
    
    races.push(race_result);

    //Set max length
    let max: usize = if let Some(_) = car_id {
        MAX_CAR_RECENT_RACES
    } else if let Some(_) = track_id {
        MAX_TRACK_RECENT_RACES
    } else {
        return Err(StdError::generic_err("No car or track ID provided"));
    };
    
    
    // Keep only the most recent races
    if races.len() > max {
        races = races.into_iter().rev().take(max).collect();
        races.reverse();
    }
    
    if let Some(car_id) = car_id {
        CAR_RECENT_RACES.save(storage, &car_id, &races)?;
    } else if let Some(track_id) = track_id {
        TRACK_RECENT_RACES.save(storage, &track_id, &races)?;
    } else {
        return Err(StdError::generic_err("No car or track ID provided"));
    }
    
    Ok(())
}
