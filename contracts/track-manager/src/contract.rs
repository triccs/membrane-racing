
// track_manager/src/contract.rs

use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult
};
use racing::race_engine::DEFAULT_SPEED;

use crate::error::TrackManagerError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{get_track, set_track, Track, ADMIN, TRACKS, TRACK_ID_COUNTER};
use racing::types::{TrackTile, TrackInfo, TileProperties};

const MAX_LIMIT: u32 = 32;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, TrackManagerError> {
    let admin = deps.api.addr_validate(&msg.admin)?;
    ADMIN.save(deps.storage, &admin)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("admin", admin))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, TrackManagerError> {
    match msg {
        ExecuteMsg::AddTrack {
            name,
            width,
            height,
            layout,
        } => execute_add_track(deps, _info, name, width, height, layout),
    }
}

pub fn execute_add_track(
    deps: DepsMut,
    _info: MessageInfo,
    name: String,
    width: u8,
    height: u8,
    layout: Vec<Vec<TileProperties>>,
) -> Result<Response, TrackManagerError> {
    // Validate track dimensions
    if width == 0 || height == 0 {
        return Err(TrackManagerError::InvalidTrackDimensions { width, height });
    }

    //Generate a new track id
    let track_id = TRACK_ID_COUNTER.load(deps.storage)?;
    TRACK_ID_COUNTER.save(deps.storage, &(track_id + Uint128::one()))?;

    // Check if track already exists
    // if get_track(deps.storage, &track_id).is_ok() {
    //     return Err(TrackManagerError::TrackAlreadyExists { track_id: track_id.clone() });
    // }

    // Validate layout dimensions
    if layout.len() != height as usize {
        return Err(TrackManagerError::InvalidTrackDimensions { width, height });
    }

    for row in &layout {
        if row.len() != width as usize {
            return Err(TrackManagerError::InvalidTrackDimensions { width, height });
        }
    }

    // Validate track layout
    validate_track_layout(&layout, width, height)?;

    // Calculate progress_towards_finish using A* pathfinding
    let track_layout = calculate_progress_towards_finish(&layout, width, height);

    // Calculate track statistics
    let stats = calculate_track_statistics(&layout, width, height);

    let track = Track {
        creator: _info.sender.to_string(),
        id: track_id.clone(),
        name,
        width,
        height,
        layout: track_layout,
    };

    set_track(deps.storage, &track_id, track)?;

    Ok(Response::new()
        .add_attribute("method", "add_track")
        .add_attribute("track_id", track_id)
        .add_attribute("width", width.to_string())
        .add_attribute("height", height.to_string())
        .add_attribute("finish_tiles", stats.finish_tiles.to_string())
        .add_attribute("boost_tiles", stats.boost_tiles.to_string())
        .add_attribute("slow_tiles", stats.slow_tiles.to_string())
        .add_attribute("stick_tiles", stats.stick_tiles.to_string())
        .add_attribute("wall_tiles", stats.wall_tiles.to_string()))
}

/// Track statistics for validation and analysis
struct TrackStats {
    finish_tiles: u32,
    boost_tiles: u32,
    slow_tiles: u32,
    stick_tiles: u32,
    wall_tiles: u32,
    normal_tiles: u32,
}

/// Calculate statistics for a track layout
fn calculate_track_statistics(
    layout: &Vec<Vec<TileProperties>>,
    width: u8,
    height: u8,
) -> TrackStats {
    let mut stats = TrackStats {
        finish_tiles: 0,
        boost_tiles: 0,
        slow_tiles: 0,
        stick_tiles: 0,
        wall_tiles: 0,
        normal_tiles: 0,
    };

    for y in 0..height {
        for x in 0..width {
            let tile = &layout[y as usize][x as usize];
            if tile.is_finish {
                stats.finish_tiles += 1;
            } else if tile.speed_modifier > DEFAULT_SPEED.into() {
                stats.boost_tiles += 1;
            } else if tile.speed_modifier < DEFAULT_SPEED.into() {
                stats.slow_tiles += 1;
            } else if tile.skip_next_turn {
                stats.stick_tiles += 1;
            } else if tile.blocks_movement {
                stats.wall_tiles += 1;
            } else {
                stats.normal_tiles += 1;
            }
        }
    }

    stats
}

/// Validate track layout for basic requirements
fn validate_track_layout(
    layout: &Vec<Vec<TileProperties>>,
    width: u8,
    height: u8,
) -> Result<(), TrackManagerError> {
    // Check for at least one finish tile
    let has_finish = layout.iter().any(|row| row.iter().any(|tile| tile.is_finish));
    if !has_finish {
        return Err(TrackManagerError::NoFinishTile {});
    }

    // Check for at least one start tile
    let has_start = layout.iter().any(|row| row.iter().any(|tile| tile.is_start));
    if !has_start {
        return Err(TrackManagerError::NoStartTile {});
    }

    // Check that every start tile can reach a finish tile
    for y in 0..height {
        for x in 0..width {
            if layout[y as usize][x as usize].is_start {
                if !can_reach_finish(layout, x, y, width, height) {
                    return Err(TrackManagerError::NoAccessiblePath {});
                }
            }
        }
    }

    // // Additional validation: ensure track is not too small or too large

    //Leave this commented out bc we don't know if small tracks could be useful for training
    // if width < 3 || height < 3 {
    //     return Err(TrackManagerError::TrackTooSmall { width, height });
    // }

    //Leave this commented out bc we don't know how big the track can be
    // if width > 50 || height > 50 {
    //     return Err(TrackManagerError::TrackTooLarge { width, height });
    // }

    Ok(())
}

/// Check if a tile can reach a finish tile using simple pathfinding
fn can_reach_finish(
    layout: &Vec<Vec<TileProperties>>,
    start_x: u8,
    start_y: u8,
    width: u8,
    height: u8,
) -> bool {
    let mut visited = vec![vec![false; width as usize]; height as usize];
    let mut queue = std::collections::VecDeque::new();
    
    queue.push_back((start_x, start_y));
    visited[start_y as usize][start_x as usize] = true;
    
    while let Some((x, y)) = queue.pop_front() {
        // Check if we reached a finish tile
        if layout[y as usize][x as usize].is_finish {
            return true;
        }
        
        // Check all 4 directions
        let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)];
        for (dx, dy) in directions {
            let nx = x as i8 + dx;
            let ny = y as i8 + dy;
            
            if nx >= 0 && ny >= 0 && nx < width as i8 && ny < height as i8 {
                let nx = nx as u8;
                let ny = ny as u8;
                
                // Skip walls and already visited tiles
                if layout[ny as usize][nx as usize].blocks_movement || visited[ny as usize][nx as usize] {
                    continue;
                }
                
                visited[ny as usize][nx as usize] = true;
                queue.push_back((nx, ny));
            }
        }
    }
    
    false
}

/// Calculate progress towards finish for each tile using A* pathfinding
fn calculate_progress_towards_finish(
    layout: &Vec<Vec<TileProperties>>,
    width: u8,
    height: u8,
) -> Vec<Vec<TrackTile>> {
    // Use A* pathfinding for more accurate distance calculation
    let distances = calculate_distances_with_astar(layout, width, height);
    
    // Convert to TrackTile format
    let mut track_layout = vec![];
    for y in 0..height {
        let mut row = vec![];
        for x in 0..width {
            let properties = layout[y as usize][x as usize].clone();
            let distance = distances[y as usize][x as usize];
            
            row.push(TrackTile {
                properties,
                progress_towards_finish: distance,
                x,
                y,
            });
        }
        track_layout.push(row);
    }
    
    track_layout
}

/// Calculate distances using A* pathfinding (more accurate than BFS)
fn calculate_distances_with_astar(
    layout: &Vec<Vec<TileProperties>>,
    width: u8,
    height: u8,
) -> Vec<Vec<u16>> {
    let mut distances = vec![vec![u16::MAX; width as usize]; height as usize];
    
    // Find all finish tiles
    let mut finish_positions = vec![];
    for y in 0..height {
        for x in 0..width {
            if layout[y as usize][x as usize].is_finish {
                finish_positions.push((x, y));
                distances[y as usize][x as usize] = 0;
            }
        }
    }
    
    // Calculate shortest path from each tile to any finish tile
    for y in 0..height {
        for x in 0..width {
            // Skip finish tiles (already set to 0)
            if layout[y as usize][x as usize].is_finish {
                continue;
            }
            
            // Skip walls (unreachable)
            if layout[y as usize][x as usize].blocks_movement {
                continue;
            }
            
            // Find shortest distance to any finish tile using A*
            let mut min_distance = u16::MAX;
            for (finish_x, finish_y) in &finish_positions {
                let distance = astar_distance(
                    x, y,
                    *finish_x, *finish_y,
                    layout, width, height
                );
                min_distance = min_distance.min(distance);
            }
            
            distances[y as usize][x as usize] = min_distance;
        }
    }
    
    distances
}

/// A* pathfinding algorithm for distance calculation
fn astar_distance(
    start_x: u8, start_y: u8,
    end_x: u8, end_y: u8,
    layout: &Vec<Vec<TileProperties>>,
    width: u8, height: u8,
) -> u16 {
    use std::collections::{BinaryHeap, HashSet};
    use std::cmp::Ordering;
    
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Node {
        x: u8,
        y: u8,
        g_cost: u16,  // Cost from start to current
        h_cost: u16,  // Heuristic cost to end
    }
    
    impl Node {
        fn f_cost(&self) -> u16 {
            self.g_cost + self.h_cost
        }
    }
    
    impl PartialOrd for Node {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.f_cost().cmp(&other.f_cost()).reverse())
        }
    }
    
    impl Ord for Node {
        fn cmp(&self, other: &Self) -> Ordering {
            self.f_cost().cmp(&other.f_cost()).reverse()
        }
    }
    
    let mut open_set = BinaryHeap::new();
    let mut closed_set = HashSet::new();
    
    // Heuristic function (Manhattan distance)
    let heuristic = |x: u8, y: u8| -> u16 {
        ((x as i16 - end_x as i16).abs() + (y as i16 - end_y as i16).abs()) as u16
    };
    
    // Start node
    open_set.push(Node {
        x: start_x,
        y: start_y,
        g_cost: 0,
        h_cost: heuristic(start_x, start_y),
    });
    
    while let Some(current) = open_set.pop() {
        // Check if we reached the end
        if current.x == end_x && current.y == end_y {
            return current.g_cost;
        }
        
        let pos = (current.x, current.y);
        if closed_set.contains(&pos) {
            continue;
        }
        closed_set.insert(pos);
        
        // Check all 4 directions
        let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)];
        
        for (dx, dy) in directions {
            let nx = current.x as i16 + dx;
            let ny = current.y as i16 + dy;
            
            // Check bounds
            if nx < 0 || ny < 0 || nx >= width as i16 || ny >= height as i16 {
                continue;
            }
            
            let nx = nx as u8;
            let ny = ny as u8;
            
            // Check if tile blocks movement
            if layout[ny as usize][nx as usize].blocks_movement {
                continue;
            }
            
            let new_g_cost = current.g_cost + 1;
            let new_h_cost = heuristic(nx, ny);
            
            open_set.push(Node {
                x: nx,
                y: ny,
                g_cost: new_g_cost,
                h_cost: new_h_cost,
            });
        }
    }
    
    // If no path found, return maximum value
    u16::MAX
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetTrack { track_id } => to_json_binary(&query_get_track(deps, track_id).map_err(|e| cosmwasm_std::StdError::generic_err(e.to_string()))?),
        QueryMsg::ListTracks {
            start_after,
            limit,
        } => to_json_binary(&query_list_tracks(deps, start_after, limit).map_err(|e| cosmwasm_std::StdError::generic_err(e.to_string()))?),
    }
}

pub fn query_get_track(deps: Deps, track_id: Uint128) -> Result<crate::msg::GetTrackResponse, TrackManagerError> {
    let track = get_track(deps.storage, &track_id)?;
    
    Ok(crate::msg::GetTrackResponse {
        track_id,
        name: track.name,
        width: track.width,
        height: track.height,
        layout: track.layout,
    })
}

pub fn query_list_tracks(deps: Deps, start_after: Option<Uint128>, limit: Option<u32>) -> Result<crate::msg::ListTracksResponse, TrackManagerError> {
    let mut tracks = vec![];
    let start_after = start_after.unwrap_or(0);
    let limit = limit.unwrap_or(MAX_LIMIT);

    for (track_id, track) in TRACKS
        .range(deps.storage, Some(&start_after), None, Order::Ascending)
        .take(limit) {

        tracks.push(TrackInfo {
            track_id,
            name: track.name,
            width: track.width,
            height: track.height,
        });
    }
    Ok(crate::msg::ListTracksResponse { tracks })
}
