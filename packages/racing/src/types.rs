use cosmwasm_schema::cw_serde;

// ===== SHARED TYPES =====

#[cw_serde]
pub struct CarMetadata {
    /// Name of the car
    pub name: String,
    /// Optional description of the car
    pub description: Option<String>,
    /// Optional URI to the car's image
    pub image_uri: Option<String>,
    /// Optional list of car attributes/traits
    pub attributes: Option<Vec<CarAttribute>>,
}

#[cw_serde]
pub struct CarAttribute {
    /// Type of the attribute (e.g., "Speed", "Handling", "Durability")
    pub trait_type: String,
    /// Value of the attribute (e.g., "High", "Medium", "Low")
    pub value: String,
}

#[cw_serde]
pub struct QTableEntry {
    /// Hash representing the state of the car
    pub state_hash:  [u8; 32],
    /// Q-values for all 4 actions [Up, Down, Left, Right]
    pub action_values: [i32; 4],
}

#[cw_serde]
pub enum RewardType {
    /// Distance-based reward with specific value
    Distance(i32),
    /// Penalty for getting stuck (negative reward)
    Stuck,
    /// Penalty for hitting a wall (negative reward)
    Wall,
    /// Penalty for no movement (negative reward)
    NoMove,
    /// Bonus for exploration (positive reward)
    Explore,
    /// Rank-based reward (0=1st place, 1=2nd place, etc.)
    Rank(u8),
}


#[cw_serde]
pub struct RewardNumbers {
    /// Distance-based reward with specific value
    pub distance: i32,
    /// Penalty for getting stuck (negative reward)
    pub stuck: i32,
    /// Penalty for hitting a wall (negative reward)
    pub wall: i32,
    /// Penalty for no movement (negative reward)
    pub no_move: i32,
    /// Bonus for exploration (positive reward)
    pub explore: i32,
    /// Rank-based reward (0=1st place, 1=2nd place, etc.)
    pub rank: RankReward,
}

#[cw_serde]
pub struct RankReward {
    pub first: i32,
    pub second: i32,
    pub third: i32,
    pub other: i32,
}

#[cw_serde]
pub struct QUpdate {
    /// Unique identifier for the car being trained
    pub car_id: String,
    /// Hash representing the current state of the car
    pub state_hash:  [u8; 32],
    /// Action taken (0=Up, 1=Down, 2=Left, 3=Right)
    pub action: u8,
    /// Type and value of reward received for this action
    pub reward_type: RewardType,
    /// Hash of the next state (None if terminal state)
    pub next_state_hash: Option< [u8; 32]>,
}

#[cw_serde]
pub struct TileProperties {
    /// Speed modifier (2 = normal, 1 = slow, 3 = boost, etc.)
    pub speed_modifier: u32,
    /// Whether this tile blocks movement
    pub blocks_movement: bool,
    /// Whether this tile causes the car to skip the next turn
    pub skip_next_turn: bool,
    /// Damage dealt to car when entering this tile (negative for healing)
    pub damage: i32,
    /// Whether this tile is a finish line
    pub is_finish: bool,
    /// Whether this tile is a start line
    pub is_start: bool,
}

impl Default for TileProperties {
    fn default() -> Self {
        Self {
            speed_modifier: 1, 
            blocks_movement: false,
            skip_next_turn: false,
            damage: 0,
            is_finish: false,
            is_start: false,
        }
    }
}

impl TileProperties {
    /// Create a normal tile
    pub fn normal() -> Self {
        Self {
            ..Default::default()
        }
    }

    /// Create a boost tile
    pub fn boost(speed_modifier: u32) -> Self {
        Self {
            speed_modifier,
            ..Default::default()
        }
    }

    //No more slow tiles bc normal speed is 1
    /// Create a slow tile
    // pub fn slow(speed_modifier: u32) -> Self {
    //     Self {
    //         speed_modifier,
    //         ..Default::default()
    //     }
    // }

    /// Create a sticky tile
    pub fn sticky() -> Self {
        Self {
            skip_next_turn: true,
            ..Default::default()
        }
    }

    /// Create a wall tile
    pub fn wall() -> Self {
        Self {
            blocks_movement: true,
            ..Default::default()
        }
    }

    /// Create a finish tile
    pub fn finish() -> Self {
        Self {
            is_finish: true,
            ..Default::default()
        }
    }

    /// Create a start tile
    pub fn start() -> Self {
        Self {
            is_start: true,
            ..Default::default()
        }
    }

    /// Create a damage tile (e.g., spikes)
    pub fn damage(damage_amount: i32) -> Self {
        Self {
            damage: damage_amount,
            ..Default::default()
        }
    }

    /// Create a healing tile
    pub fn healing(heal_amount: i32) -> Self {
        Self {
            damage: -heal_amount, // Negative damage = healing
            ..Default::default()
        }
    }

    /// Get the effective speed modifier for this tile
    pub fn get_speed_modifier(&self) -> u32 {
        self.speed_modifier
    }

    /// Check if this tile blocks movement
    pub fn blocks_movement(&self) -> bool {
        self.blocks_movement
    }

    /// Check if this tile causes the car to skip the next turn
    pub fn skip_next_turn(&self) -> bool {
        self.skip_next_turn
    }

    /// Get the damage/healing amount for this tile
    pub fn get_damage(&self) -> i32 {
        self.damage
    }

    /// Check if this tile is a finish line
    pub fn is_finish(&self) -> bool {
        self.is_finish
    }

    /// Check if this tile is a start line
    pub fn is_start(&self) -> bool {
        self.is_start
    }
}

#[cw_serde]
pub struct TrackTile {
    /// Properties of the track tile
    pub properties: TileProperties,
    /// Progress towards the finish line in positions
    pub progress_towards_finish: u16,
    /// x position of the tile
    pub x: u8,
    /// y position of the tile
    pub y: u8,
}

#[cw_serde]
pub struct Track {
    /// Unique identifier for the track
    pub id: u64,
    /// Name of the track
    pub name: String,
    /// Width of the track in tiles
    pub width: u8,
    /// Height of the track in tiles
    pub height: u8,
    /// 2D layout of the track with tile information
    pub layout: Vec<Vec<TrackTile>>,
}

#[cw_serde]
pub struct TrackInfo {
    /// Unique identifier for the track
    pub track_id: String,
    /// Name of the track
    pub name: String,
    /// Width of the track in tiles
    pub width: u8,
    /// Height of the track in tiles
    pub height: u8,
}

#[cw_serde]
pub enum TournamentCriteria {
    /// Random selection of cars
    Random,
    /// Top trained cars with minimum training updates
    TopTrained { 
        /// Minimum number of training updates required
        min_training_updates: u32 
    },
    /// All cars participate
    AllCars,
}

#[cw_serde]
pub enum TournamentStatus {
    /// Tournament has not started yet
    NotStarted,
    /// Tournament is currently in progress
    InProgress,
    /// Tournament has completed
    Completed,
}

#[cw_serde]
pub struct TournamentMatch {
    /// Unique identifier for the match
    pub match_id: String,
    /// First car in the match
    pub car1: String,
    /// Second car in the match
    pub car2: String,
    /// Winner of the match (None if not completed)
    pub winner: Option<String>,
    /// Whether the match has been completed
    pub completed: bool,
}

#[cw_serde]
pub struct TournamentResult {
    /// Unique identifier for the car
    pub car_id: String,
    /// Final rank in the tournament
    pub rank: u32,
    /// Number of wins in the tournament
    pub wins: u32,
    /// Number of losses in the tournament
    pub losses: u32,
}

#[cw_serde]
pub struct TournamentRanking {
    /// Unique identifier for the car
    pub car_id: String,
    /// Final rank in the tournament
    pub rank: u32,
    /// Number of wins in the tournament
    pub wins: u32,
    /// Number of losses in the tournament
    pub losses: u32,
}


/// Strategies for selecting actions during training or racing
#[cw_serde]
pub enum ActionSelectionStrategy {
    Best,                       // Exploit: highest Q-value
    Random,                     // Pure exploration
    EpsilonGreedy(f32),         // Exploration with ε chance
    Softmax(f32),               // Probabilistic based on Q-values
    EpsilonDecay {              // Epsilon that decays over training progress
        initial_epsilon: f32,   // Starting epsilon value
        final_epsilon: f32,     // Final epsilon value
        current_tick: u32,      // Current training tick
        total_ticks: u32,       // Total training ticks
    },
}

// Example usage of the new TileProperties system:
#[cfg(test)]
mod tests {
    use super::*;
    use crate::race_engine::DEFAULT_BOOST_SPEED;

    #[test]
    fn test_tile_properties_creation() {
        // Basic tiles
        let normal = TileProperties::normal();
        assert_eq!(normal.speed_modifier, 2);
        assert!(!normal.blocks_movement);

        let boost = TileProperties::boost(DEFAULT_BOOST_SPEED as u32);
        assert_eq!(boost.speed_modifier, DEFAULT_BOOST_SPEED as u32);
        assert!(!boost.blocks_movement);

        let wall = TileProperties::wall();
        assert!(wall.blocks_movement);

        let sticky = TileProperties::sticky();
        assert!(sticky.skip_next_turn);

        let finish = TileProperties::finish();
        assert!(finish.is_finish);

        let start = TileProperties::start();
        assert!(start.is_start);
    }

    #[test]
    fn test_damage_and_healing() {
        // Damage tile
        let spikes = TileProperties::damage(10);
        assert_eq!(spikes.get_damage(), 10);

        // Healing tile
        let healing = TileProperties::healing(5);
        assert_eq!(healing.get_damage(), -5);
    }

    #[test]
    fn test_property_combinations() {
        // A boost tile that also heals
        let boost_heal = TileProperties::boost(DEFAULT_BOOST_SPEED as u32);
        assert_eq!(boost_heal.get_speed_modifier(), DEFAULT_BOOST_SPEED as u32);
        assert_eq!(boost_heal.get_damage(), 0);

        // A wall that blocks movement
        let wall = TileProperties::wall();
        assert!(wall.blocks_movement);
    }

    #[test]
    fn test_sticky_combinations() {
        // A tile that's sticky
        let sticky = TileProperties::sticky();
        assert!(sticky.skip_next_turn);
        assert_eq!(sticky.get_speed_modifier(), 2); // Default speed
    }
}

/*
Example usage in track creation:

```rust
// Create a complex track with various tile types
let track_layout = vec![
    vec![
        TrackTile {
            properties: TileProperties::start(),
            progress_towards_finish: 10,
            x: 0, y: 0,
        },
        TrackTile {
            properties: TileProperties::normal(),
            progress_towards_finish: 9,
            x: 1, y: 0,
        },
        TrackTile {
            properties: TileProperties::boost(2.0),
            progress_towards_finish: 8,
            x: 2, y: 0,
        },
        TrackTile {
            properties: TileProperties::damage(5),
            progress_towards_finish: 7,
            x: 3, y: 0,
        },
        TrackTile {
            properties: TileProperties::teleporter(0, 5),
            progress_towards_finish: 6,
            x: 4, y: 0,
        },
    ],
    vec![
        TrackTile {
            properties: TileProperties::wall(),
            progress_towards_finish: 9,
            x: 0, y: 1,
        },
        TrackTile {
            properties: TileProperties::slow(0.5),
            progress_towards_finish: 8,
            x: 1, y: 1,
        },
        TrackTile {
            properties: TileProperties::sticky(),
            progress_towards_finish: 7,
            x: 2, y: 1,
        },
        TrackTile {
            properties: TileProperties::healing(3),
            progress_towards_finish: 6,
            x: 3, y: 1,
        },
        TrackTile {
            properties: TileProperties::checkpoint(1),
            progress_towards_finish: 5,
            x: 4, y: 1,
        },
    ],
    // ... more rows
    vec![
        TrackTile {
            properties: TileProperties::finish(),
            progress_towards_finish: 0,
            x: 0, y: 5,
        },
        TrackTile {
            properties: TileProperties::finish(),
            progress_towards_finish: 0,
            x: 1, y: 5,
        },
        TrackTile {
            properties: TileProperties::finish(),
            progress_towards_finish: 0,
            x: 2, y: 5,
        },
        TrackTile {
            properties: TileProperties::finish(),
            progress_towards_finish: 0,
            x: 3, y: 5,
        },
        TrackTile {
            properties: TileProperties::finish(),
            progress_towards_finish: 0,
            x: 4, y: 5,
        },
    ],
];

// Example of complex tile combinations:
let complex_tile = TileProperties::boost(2.5)
    .with_custom_property("damage".to_string(), "3".to_string())
    .with_custom_property("teleporter".to_string(), "true".to_string())
    .with_custom_property("dest_x".to_string(), "5".to_string())
    .with_custom_property("dest_y".to_string(), "10".to_string());

// This tile is:
// - A boost tile (2.5x speed)
// - Deals 3 damage
// - Teleports to (5, 10)
```

This new properties-only system provides:
1. **Pure property-based design**: No enum constraints, everything is a property
2. **Infinite combinations**: Mix and match any properties you want
3. **Extensibility**: Add new properties without changing the core structure
4. **Type safety**: All properties are strongly typed
5. **Flexibility**: Create complex tiles like "boost + damage + teleporter"
6. **Clean API**: Simple helper methods for common tile types
7. **Custom properties**: Add any key-value pairs for special effects

The system is now completely property-driven, allowing for maximum flexibility in tile design!
*/
