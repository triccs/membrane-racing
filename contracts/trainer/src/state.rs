use cosmwasm_std::{Addr, StdResult, Storage};
use cw_storage_plus::{Item, Map};
use racing::trainer::TrainingSession;

pub const ADMIN: Item<Addr> = Item::new("admin");

// Q-table storage: (car_id, state_hash) -> [i32; 5] action values
pub const Q_TABLE: Map<(&str, &str), [i32; 5]> = Map::new("q_table");

// Training statistics: car_id -> number of updates
pub const TRAINING_STATS: Map<&str, u32> = Map::new("training_stats");

// Track results: (car_id, track_id) -> (wins, losses)
pub const TRACK_RESULTS: Map<(&str, &str), (u32, u32)> = Map::new("track_results");

// **NEW**: Active training sessions storage
pub const ACTIVE_TRAINING_SESSIONS: Map<&str, TrainingSession> = Map::new("active_training_sessions");

pub fn get_q_values(storage: &dyn Storage, car_id: &str, state_hash: &str) -> StdResult<[i32; 5]> {
    Q_TABLE.load(storage, (car_id, state_hash))
}

pub fn set_q_values(
    storage: &mut dyn Storage,
    car_id: &str,
    state_hash: &str,
    q_values: [i32; 5],
) -> StdResult<()> {
    Q_TABLE.save(storage, (car_id, state_hash), &q_values)
}

pub fn get_training_stats(storage: &dyn Storage, car_id: &str) -> StdResult<u32> {
    TRAINING_STATS.load(storage, car_id)
}

pub fn increment_training_stats(storage: &mut dyn Storage, car_id: &str) -> StdResult<u32> {
    let current = TRAINING_STATS.load(storage, car_id).unwrap_or(0);
    let new_count = current + 1;
    TRAINING_STATS.save(storage, car_id, &new_count)?;
    Ok(new_count)
}

pub fn get_track_results(storage: &dyn Storage, car_id: &str, track_id: &str) -> StdResult<(u32, u32)> {
    TRACK_RESULTS.load(storage, (car_id, track_id))
}

pub fn update_track_results(
    storage: &mut dyn Storage,
    car_id: &str,
    track_id: &str,
    won: bool,
) -> StdResult<(u32, u32)> {
    let (mut wins, mut losses) = TRACK_RESULTS.load(storage, (car_id, track_id)).unwrap_or((0, 0));
    
    if won {
        wins += 1;
    } else {
        losses += 1;
    }
    
    TRACK_RESULTS.save(storage, (car_id, track_id), &(wins, losses))?;
    Ok((wins, losses))
}

// **NEW**: Training session state management
pub fn save_training_session(storage: &mut dyn Storage, car_id: &str, session: &TrainingSession) -> StdResult<()> {
    ACTIVE_TRAINING_SESSIONS.save(storage, car_id, session)
}

pub fn get_training_session(storage: &dyn Storage, car_id: &str) -> StdResult<TrainingSession> {
    ACTIVE_TRAINING_SESSIONS.load(storage, car_id)
}

pub fn remove_training_session(storage: &mut dyn Storage, car_id: &str) -> StdResult<()> {
    ACTIVE_TRAINING_SESSIONS.remove(storage, car_id);
    Ok(())
} 