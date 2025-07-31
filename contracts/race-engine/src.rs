// race_engine/src/contract.rs

use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo, Response, StdResult, Binary, to_binary, attr};
use crate::msg::{ExecuteMsg, QueryMsg, RaceResult, SimulateRaceParams};
use crate::state::{RoundState, CarPosition};
use track_manager::state::{TileType, TrackTile, Track, TRACKS};
use car_nft::state::Q_TABLE;
use std::collections::{HashMap, HashSet};

pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: (),
) -> StdResult<Response> {
    Ok(Response::default())
}

pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::SimulateRace { params } => simulate_race(deps, params),
    }
}

pub fn query(
    _deps: Deps,
    _env: Env,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetRaceResult { race_id: _ } => to_binary("Deprecated: race results are not stored."),
    }
}

fn simulate_race(
    deps: DepsMut,
    params: SimulateRaceParams,
) -> StdResult<Response> {
    let track: Track = TRACKS.load(deps.storage, params.track_id)?;

    let mut positions: HashMap<String, (u8, u8)> = params.car_ids.iter().enumerate().map(|(i, id)| {
        (id.clone(), (i as u8 % track.width, 0))
    }).collect();

    let mut stuck_flags: HashMap<String, bool> = params.car_ids.iter().map(|id| (id.clone(), false)).collect();
    let mut finished: HashSet<String> = HashSet::new();
    let mut steps_taken: HashMap<String, u32> = params.car_ids.iter().map(|id| (id.clone(), 0)).collect();

    let mut play_by_play = vec![];
    let mut tick = 0;
    let max_ticks = 100;

    // Load full Q-table for each car
    let mut q_tables: HashMap<String, HashMap<String, [i32; 5]>> = HashMap::new();
    for car_id in &params.car_ids {
        let prefix = car_id.clone();
        let mut car_q_table = HashMap::new();
        let q_entries = deps.querier.query::<Vec<PriceResponse>>(
                    &QueryRequest::Wasm(WasmQuery::Smart {
                        contract_addr: config.car_nft.clone(),
                        msg: to_json_binary(&Car_QueryMsg::GetQ {
                          car_id,
                          state_hash: None
                        })?,
                    })?;
      
        for item in q_entries {
            let ((_, state_hash), q_vals) = item?;
            car_q_table.insert(state_hash, q_vals);
        }
        q_tables.insert(car_id.clone(), car_q_table);
    }

    while tick < max_ticks && finished.len() < params.car_ids.len() {
        let mut destination_map: HashMap<(u8, u8), Vec<String>> = HashMap::new();
        let mut proposed_positions: HashMap<String, (u8, u8)> = HashMap::new();

        for car_id in &params.car_ids {
            let (x, y) = positions[car_id];
            let tile = track.layout[y as usize][x as usize];

            if stuck_flags[car_id] {
                stuck_flags.insert(car_id.clone(), false);
                proposed_positions.insert(car_id.clone(), (x, y));
                continue;
            }

            let mut sensors = Vec::with_capacity(9);
            sensors.push(tile.tile_type.to_string());

            for dy in -1i8..=1 {
                for dx in -1i8..=1 {
                    if dx == 0 && dy == 0 { continue; }
                    let nx = x as i8 + dx;
                    let ny = y as i8 + dy;
                    let tile_type = if nx >= 0 && ny >= 0 &&
                        (nx as usize) < track.width as usize &&
                        (ny as usize) < track.height as usize {
                        track.layout[ny as usize][nx as usize].tile_type.to_string()
                    } else {
                        "Wall".to_string()
                    };
                    sensors.push(tile_type);
                }
            }

            let state_hash = sensors.join(",");
            let q_vals = q_tables[car_id].get(&state_hash).cloned().unwrap_or([0; 5]);
            let best_action = q_vals.iter().enumerate().max_by_key(|(_, val)| *val).map(|(i, _)| i).unwrap_or(1);

            let move_distance = match best_action {
                0 => 0,
                1 => 1,
                2 => 2,
                3 => 3,
                _ => 1,
            };

            let target_y = (y + move_distance.min(3)).min(track.height - 1);
            let next_tile = track.layout[target_y as usize][x as usize];
            let next_pos = if next_tile.tile_type == TileType::Wall {
                (x, y)
            } else {
                (x, target_y)
            };

            proposed_positions.insert(car_id.clone(), next_pos);
        }

        for (car_id, pos) in &proposed_positions {
            destination_map.entry(*pos).or_default().push(car_id.clone());
        }

        for car_id in &params.car_ids {
            if finished.contains(car_id) {
                continue;
            }

            let new_pos = proposed_positions[car_id];
            if destination_map[&new_pos].len() > 1 {
                continue;
            }

            positions.insert(car_id.clone(), new_pos);
            steps_taken.entry(car_id.clone()).and_modify(|v| *v += 1);

            let tile = track.layout[new_pos.1 as usize][new_pos.0 as usize];
            if tile.tile_type == TileType::Stick {
                stuck_flags.insert(car_id.clone(), true);
            }
            if tile.tile_type == TileType::Finish {
                finished.insert(car_id.clone());
            }
        }

        let car_positions = params.car_ids.iter().map(|id| {
            let (x, y) = positions[id];
            CarPosition {
                car_id: id.clone(),
                x,
                y,
                stuck: stuck_flags[id],
            }
        }).collect();

        play_by_play.push(RoundState {
            tick,
            car_positions,
        });

        tick += 1;
    }

    let mut cars: Vec<_> = params.car_ids.clone();
    cars.sort_by_key(|id| {
        if finished.contains(id) {
            (0, steps_taken[id])
        } else {
            let (x, y) = positions[id];
            let dist = track.layout[y as usize][x as usize].distance_from_finish;
            (1, dist as u32)
        }
    });

    let winner_ids = if finished.is_empty() {
        vec![]
    } else {
        vec![cars[0].clone()]
    };

    let rankings = cars.iter().map(|id| (id.clone(), steps_taken[id])).collect();

    let result = RaceResult {
        winner_ids,
        rankings,
        play_by_play,
    };

    let json = serde_json::to_string(&result)?;

    Ok(Response::new()
        .add_attribute("action", "simulate_race")
        .add_attribute("result", json))
}
