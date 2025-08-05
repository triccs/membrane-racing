use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coins, from_json, Addr};
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

use crate::contract::{execute, instantiate, query};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use racing::types::{RewardType, QUpdate};

// Mock race engine contract for integration testing
fn mock_race_engine_contract() -> Box<dyn Contract<cosmwasm_std::Empty>> {
    let contract = ContractWrapper::new(
        |deps, env, info, msg: racing::race_engine::ExecuteMsg| {
            match msg {
                racing::race_engine::ExecuteMsg::SimulateRace { track_id, car_ids } => {
                    // Mock race simulation that returns training data
                    Ok::<cosmwasm_std::Response, cosmwasm_std::StdError>(
                        cosmwasm_std::Response::new()
                            .add_attribute("method", "simulate_race")
                            .add_attribute("track_id", track_id)
                            .add_attribute("car_count", car_ids.len().to_string())
                    )
                }
            }
        },
        |deps, env, info, msg: racing::race_engine::InstantiateMsg| {
            Ok::<cosmwasm_std::Response, cosmwasm_std::StdError>(cosmwasm_std::Response::new())
        },
        |deps, env, msg: racing::race_engine::QueryMsg| {
            match msg {
                racing::race_engine::QueryMsg::GetRaceResult { race_id } => {
                    // Return mock race result with training data
                    let result = racing::race_engine::RaceResult {
                        race_id,
                        track_id: "test_track".to_string(),
                        car_ids: vec!["test_car".to_string()],
                        winner_ids: vec!["test_car".to_string()],
                        rankings: vec![("test_car".to_string(), 10)],
                        play_by_play: vec!["Tick 0".to_string(), "Tick 1".to_string()],
                        steps_taken: vec![("test_car".to_string(), 10)],
                    };
                    Ok(cosmwasm_std::to_json_binary(&result).unwrap())
                },
                _ => Err(cosmwasm_std::StdError::generic_err("Not implemented")),
            }
        },
    );
    Box::new(contract)
}

fn trainer_contract() -> Box<dyn Contract<cosmwasm_std::Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

fn setup_test_app() -> (App, Addr, Addr) {
    let mut app = AppBuilder::new().build(|router, _, storage| {
        router
            .bank
            .init_balance(storage, &Addr::unchecked("owner"), coins(1000, "earth"))
            .unwrap();
    });

    let race_engine_contract_id = app.store_code(mock_race_engine_contract());
    let trainer_contract_id = app.store_code(trainer_contract());

    let race_engine_addr = app
        .instantiate_contract(
            race_engine_contract_id,
            Addr::unchecked("owner"),
            &racing::race_engine::InstantiateMsg { admin: "owner".to_string() },
            &[],
            "Race Engine",
            None,
        )
        .unwrap();

    let trainer_addr = app
        .instantiate_contract(
            trainer_contract_id,
            Addr::unchecked("owner"),
            &InstantiateMsg { admin: "owner".to_string() },
            &[],
            "Trainer",
            None,
        )
        .unwrap();

    (app, race_engine_addr, trainer_addr)
}

// Helper function to create training data for different scenarios
fn create_training_data_for_track(track_type: &str, training_rounds: u32) -> Vec<QUpdate> {
    let mut updates = Vec::new();
    
    for round in 0..training_rounds {
        for step in 0..10 {
            let state_hash = format!("{}_{}_{}", track_type, round, step);
            let next_state_hash = if step < 9 {
                Some(format!("{}_{}_{}", track_type, round, step + 1))
            } else {
                None
            };
            
            // Different reward types based on track and progress
            let reward_type = match track_type {
                "straight" => {
                    if step < 5 {
                        RewardType::Distance(10) // Moving forward
                    } else {
                        RewardType::Distance(20) // Closer to finish
                    }
                },
                "zigzag" => {
                    if step % 2 == 0 {
                        RewardType::Distance(15) // Good turn
                    } else {
                        RewardType::Distance(5) // Straight section
                    }
                },
                "special" => {
                    match step {
                        0..=2 => RewardType::Distance(10), // Normal movement
                        3 => RewardType::Explore, // Boost tile effect
                        4 => RewardType::Distance(5), // Slow tile effect
                        5 => RewardType::Stuck, // Stuck tile
                        6 => RewardType::Wall, // Wall collision
                        7..=9 => RewardType::Distance(15), // Recovery
                        _ => RewardType::Distance(5),
                    }
                },
                _ => RewardType::Distance(10),
            };
            
            updates.push(QUpdate {
                car_id: "test_car".to_string(),
                state_hash,
                action: (step % 5) as u8, // Cycle through actions
                reward_type,
                next_state_hash,
            });
        }
    }
    
    updates
}

#[test]
fn test_basic_training_single_track() {
    let (mut app, _race_engine_addr, trainer_addr) = setup_test_app();

    let car_id = "test_car".to_string();
    let _track_id = "straight_track".to_string();

    // Initial training stats
    let initial_stats: crate::msg::GetCarTrainingStatsResponse = app
        .wrap()
        .query_wasm_smart(&trainer_addr, &QueryMsg::GetCarTrainingStats { car_id: car_id.clone() })
        .unwrap();
    
    assert_eq!(initial_stats.training_updates, 0);

    // Perform training on straight track
    let training_data = create_training_data_for_track("straight", 5);
    
    for (_i, update) in training_data.iter().enumerate() {
        let msg = ExecuteMsg::UpdateQValue {
            car_id: update.car_id.clone(),
            state_hash: update.state_hash.clone(),
            action: update.action,
            reward_type: update.reward_type.clone(),
            next_state_hash: update.next_state_hash.clone(),
        };

        let result = app
            .execute_contract(
                Addr::unchecked("owner"),
                trainer_addr.clone(),
                &msg,
                &[],
            )
            .unwrap();

        // Verify training attributes
        assert!(result.events.iter().any(|event| {
            event.ty == "wasm" && event.attributes.iter().any(|attr| {
                attr.key == "method" && attr.value == "update_q_value"
            })
        }));
    }

    // Check final training stats
    let final_stats: crate::msg::GetCarTrainingStatsResponse = app
        .wrap()
        .query_wasm_smart(&trainer_addr, &QueryMsg::GetCarTrainingStats { car_id: car_id.clone() })
        .unwrap();
    
    assert_eq!(final_stats.training_updates, 50); // 5 rounds * 10 steps

    // Check Q-values for a specific state
    let q_values: crate::msg::GetQValueResponse = app
        .wrap()
        .query_wasm_smart(&trainer_addr, &QueryMsg::GetQValue { 
            car_id: car_id.clone(), 
            state_hash: "straight_0_0".to_string() 
        })
        .unwrap();
    
    assert_eq!(q_values.q_values.len(), 5);
    assert!(q_values.q_values.iter().any(|&val| val > 0)); // Should have learned positive values
}

#[test]
fn test_batch_training_single_track() {
    let (mut app, _race_engine_addr, trainer_addr) = setup_test_app();

    let car_id = "test_car".to_string();
    let _track_id = "straight_track".to_string();

    // Create batch training data
    let training_data = create_training_data_for_track("straight", 3);
    
    let msg = ExecuteMsg::BatchUpdateQValues { updates: training_data };

    let result = app
        .execute_contract(
            Addr::unchecked("owner"),
            trainer_addr.clone(),
            &msg,
            &[],
        )
        .unwrap();

    // Verify batch training attributes
    assert!(result.events.iter().any(|event| {
        event.ty == "wasm" && event.attributes.iter().any(|attr| {
            attr.key == "method" && attr.value == "batch_update_q_values"
        })
    }));

    // Check training stats
    let stats: crate::msg::GetCarTrainingStatsResponse = app
        .wrap()
        .query_wasm_smart(&trainer_addr, &QueryMsg::GetCarTrainingStats { car_id: car_id.clone() })
        .unwrap();
    
    assert_eq!(stats.training_updates, 30); // 3 rounds * 10 steps
}

#[test]
fn test_training_progress_output() {
    let (mut app, _race_engine_addr, trainer_addr) = setup_test_app();

    let car_id = "test_car".to_string();
    let track_id = "straight_track".to_string();

    println!("\n=== Training Progress Output ===");
    println!("Car ID: {}", car_id);
    println!("Track ID: {}", track_id);
    println!("Starting training...\n");

    // Perform training with progress tracking
    let training_rounds = 5;
    let steps_per_round = 10;
    
    for round in 0..training_rounds {
        println!("--- Training Round {} ---", round + 1);
        
        for step in 0..steps_per_round {
            let state_hash = format!("straight_{}_{}", round, step);
            let next_state_hash = if step < steps_per_round - 1 {
                Some(format!("straight_{}_{}", round, step + 1))
            } else {
                None
            };
            
            let reward_type = if step < 5 {
                RewardType::Distance(10)
            } else {
                RewardType::Distance(20)
            };
            
            let msg = ExecuteMsg::UpdateQValue {
                car_id: car_id.clone(),
                state_hash: state_hash.clone(),
                action: (step % 5) as u8,
                reward_type,
                next_state_hash,
            };

            let result = app
                .execute_contract(
                    Addr::unchecked("owner"),
                    trainer_addr.clone(),
                    &msg,
                    &[],
                )
                .unwrap();

            // Extract training metrics from response
            let old_value = result.events.iter()
                .find(|event| event.ty == "wasm")
                .and_then(|event| event.attributes.iter()
                    .find(|attr| attr.key == "old_value")
                    .map(|attr| attr.value.parse::<i32>().unwrap_or(0)))
                .unwrap_or(0);

            let new_value = result.events.iter()
                .find(|event| event.ty == "wasm")
                .and_then(|event| event.attributes.iter()
                    .find(|attr| attr.key == "new_value")
                    .map(|attr| attr.value.parse::<i32>().unwrap_or(0)))
                .unwrap_or(0);

            let reward = result.events.iter()
                .find(|event| event.ty == "wasm")
                .and_then(|event| event.attributes.iter()
                    .find(|attr| attr.key == "reward")
                    .map(|attr| attr.value.parse::<i32>().unwrap_or(0)))
                .unwrap_or(0);

            println!("  Step {}: Action={}, Old={}, New={}, Reward={}", 
                    step + 1, step % 5, old_value, new_value, reward);
        }
        
        // Check training stats after each round
        let stats: crate::msg::GetCarTrainingStatsResponse = app
            .wrap()
            .query_wasm_smart(&trainer_addr, &QueryMsg::GetCarTrainingStats { car_id: car_id.clone() })
            .unwrap();
        
        println!("  Training updates: {}", stats.training_updates);
        
        // Check Q-values for first state
        let q_values: crate::msg::GetQValueResponse = app
            .wrap()
            .query_wasm_smart(&trainer_addr, &QueryMsg::GetQValue { 
                car_id: car_id.clone(), 
                state_hash: "straight_0_0".to_string() 
            })
            .unwrap();
        
        println!("  Q-values for state 'straight_0_0': {:?}", q_values.q_values);
        println!();
    }

    // Final training summary
    let final_stats: crate::msg::GetCarTrainingStatsResponse = app
        .wrap()
        .query_wasm_smart(&trainer_addr, &QueryMsg::GetCarTrainingStats { car_id: car_id.clone() })
        .unwrap();
    
    println!("=== Final Training Summary ===");
    println!("Total training updates: {}", final_stats.training_updates);
    println!("Average updates per round: {:.1}", final_stats.training_updates as f32 / training_rounds as f32);
}

#[test]
fn test_multi_track_training() {
    let (mut app, _race_engine_addr, trainer_addr) = setup_test_app();

    let car_id = "test_car".to_string();
    
    println!("\n=== Multi-Track Training Test ===");
    println!("Car ID: {}", car_id);

    // Track configurations
    let tracks = vec![
        ("straight", "Straight Track", 3),
        ("zigzag", "Zigzag Track", 3),
        ("special", "Special Tiles Track", 3),
    ];

    for (track_type, track_name, training_rounds) in &tracks {
        println!("\n--- Training on {} ---", track_name);
        
        let training_data = create_training_data_for_track(track_type, *training_rounds);
        
        for (_i, update) in training_data.iter().enumerate() {
            let msg = ExecuteMsg::UpdateQValue {
                car_id: update.car_id.clone(),
                state_hash: update.state_hash.clone(),
                action: update.action,
                reward_type: update.reward_type.clone(),
                next_state_hash: update.next_state_hash.clone(),
            };

            app.execute_contract(
                Addr::unchecked("owner"),
                trainer_addr.clone(),
                &msg,
                &[],
            ).unwrap();
        }

        // Check training stats for this track
        let stats: crate::msg::GetCarTrainingStatsResponse = app
            .wrap()
            .query_wasm_smart(&trainer_addr, &QueryMsg::GetCarTrainingStats { car_id: car_id.clone() })
            .unwrap();
        
        println!("  Training updates: {}", stats.training_updates);
        
        // Check Q-values for this track's first state
        let q_values: crate::msg::GetQValueResponse = app
            .wrap()
            .query_wasm_smart(&trainer_addr, &QueryMsg::GetQValue { 
                car_id: car_id.clone(), 
                state_hash: format!("{}_0_0", track_type)
            })
            .unwrap();
        
        println!("  Q-values for state '{}_{}_0_0': {:?}", track_type, track_type, q_values.q_values);
        
        // Record track results
        let won = track_type == &"straight"; // Simulate different results per track
        let steps_taken = 15;
        
        let track_result_msg = ExecuteMsg::RecordTrackResult {
            car_id: car_id.clone(),
            track_id: track_type.to_string(),
            won,
            steps_taken,
        };
        
        app.execute_contract(
            Addr::unchecked("owner"),
            trainer_addr.clone(),
            &track_result_msg,
            &[],
        ).unwrap();
        
        // Check track results
        let track_results: crate::msg::GetTrackResultsResponse = app
            .wrap()
            .query_wasm_smart(&trainer_addr, &QueryMsg::GetTrackResults { 
                car_id: car_id.clone(), 
                track_id: track_type.to_string() 
            })
            .unwrap();
        
        println!("  Track results: {} wins, {} losses", track_results.wins, track_results.losses);
    }

    // Final multi-track summary
    let final_stats: crate::msg::GetCarTrainingStatsResponse = app
        .wrap()
        .query_wasm_smart(&trainer_addr, &QueryMsg::GetCarTrainingStats { car_id: car_id.clone() })
        .unwrap();
    
    println!("\n=== Final Multi-Track Summary ===");
    println!("Total training updates: {}", final_stats.training_updates);
    
    // Check results for each track
    for (track_type, track_name, _) in &tracks {
        let track_results: crate::msg::GetTrackResultsResponse = app
            .wrap()
            .query_wasm_smart(&trainer_addr, &QueryMsg::GetTrackResults { 
                car_id: car_id.clone(), 
                track_id: track_type.to_string() 
            })
            .unwrap();
        
        println!("{}: {} wins, {} losses", track_name, track_results.wins, track_results.losses);
    }
}

#[test]
fn test_track_specific_training_scenarios() {
    let (mut app, _race_engine_addr, trainer_addr) = setup_test_app();

    let car_id = "test_car".to_string();
    
    println!("\n=== Track-Specific Training Scenarios ===");

    // Test 1: Straight Track Training
    println!("\n1. Straight Track Training");
    let straight_data = create_training_data_for_track("straight", 2);
    
    for update in straight_data {
        let msg = ExecuteMsg::UpdateQValue {
            car_id: update.car_id.clone(),
            state_hash: update.state_hash.clone(),
            action: update.action,
            reward_type: update.reward_type.clone(),
            next_state_hash: update.next_state_hash.clone(),
        };
        
        app.execute_contract(
            Addr::unchecked("owner"),
            trainer_addr.clone(),
            &msg,
            &[],
        ).unwrap();
    }
    
    let straight_stats: crate::msg::GetCarTrainingStatsResponse = app
        .wrap()
        .query_wasm_smart(&trainer_addr, &QueryMsg::GetCarTrainingStats { car_id: car_id.clone() })
        .unwrap();
    
    println!("   Training updates: {}", straight_stats.training_updates);
    
    // Test 2: Zigzag Track Training
    println!("\n2. Zigzag Track Training");
    let zigzag_data = create_training_data_for_track("zigzag", 2);
    
    for update in zigzag_data {
        let msg = ExecuteMsg::UpdateQValue {
            car_id: update.car_id.clone(),
            state_hash: update.state_hash.clone(),
            action: update.action,
            reward_type: update.reward_type.clone(),
            next_state_hash: update.next_state_hash.clone(),
        };
        
        app.execute_contract(
            Addr::unchecked("owner"),
            trainer_addr.clone(),
            &msg,
            &[],
        ).unwrap();
    }
    
    let zigzag_stats: crate::msg::GetCarTrainingStatsResponse = app
        .wrap()
        .query_wasm_smart(&trainer_addr, &QueryMsg::GetCarTrainingStats { car_id: car_id.clone() })
        .unwrap();
    
    println!("   Training updates: {}", zigzag_stats.training_updates);
    
    // Test 3: Special Tiles Track Training
    println!("\n3. Special Tiles Track Training");
    let special_data = create_training_data_for_track("special", 2);
    
    for update in special_data {
        let msg = ExecuteMsg::UpdateQValue {
            car_id: update.car_id.clone(),
            state_hash: update.state_hash.clone(),
            action: update.action,
            reward_type: update.reward_type.clone(),
            next_state_hash: update.next_state_hash.clone(),
        };
        
        app.execute_contract(
            Addr::unchecked("owner"),
            trainer_addr.clone(),
            &msg,
            &[],
        ).unwrap();
    }
    
    let special_stats: crate::msg::GetCarTrainingStatsResponse = app
        .wrap()
        .query_wasm_smart(&trainer_addr, &QueryMsg::GetCarTrainingStats { car_id: car_id.clone() })
        .unwrap();
    
    println!("   Training updates: {}", special_stats.training_updates);
    
    // Compare Q-values across different tracks
    println!("\n4. Q-Value Comparison Across Tracks");
    
    for track_type in &["straight", "zigzag", "special"] {
        let q_values: crate::msg::GetQValueResponse = app
            .wrap()
            .query_wasm_smart(&trainer_addr, &QueryMsg::GetQValue { 
                car_id: car_id.clone(), 
                state_hash: format!("{}_0_0", track_type)
            })
            .unwrap();
        
        println!("   {} track Q-values: {:?}", track_type, q_values.q_values);
    }
}

#[test]
fn test_reward_type_learning() {
    let (mut app, _race_engine_addr, trainer_addr) = setup_test_app();

    let car_id = "test_car".to_string();
    
    println!("\n=== Reward Type Learning Test ===");
    
    // Test different reward types
    let reward_types = vec![
        (RewardType::Distance(10), "Distance"),
        (RewardType::Stuck, "Stuck"),
        (RewardType::Wall, "Wall"),
        (RewardType::NoMove, "NoMove"),
        (RewardType::Explore, "Explore"),
        (RewardType::Rank(0), "Rank 1st"),
        (RewardType::Rank(1), "Rank 2nd"),
        (RewardType::Rank(2), "Rank 3rd"),
    ];
    
    for (i, (reward_type, reward_name)) in reward_types.iter().enumerate() {
        let state_hash = format!("reward_test_{}", i);
        let next_state_hash = if i < reward_types.len() - 1 {
            Some(format!("reward_test_{}", i + 1))
        } else {
            None
        };
        
        let msg = ExecuteMsg::UpdateQValue {
            car_id: car_id.clone(),
            state_hash: state_hash.clone(),
            action: (i % 5) as u8,
            reward_type: reward_type.clone(),
            next_state_hash,
        };
        
        let result = app
            .execute_contract(
                Addr::unchecked("owner"),
                trainer_addr.clone(),
                &msg,
                &[],
            )
            .unwrap();
        
        // Extract reward value from response
        let reward = result.events.iter()
            .find(|event| event.ty == "wasm")
            .and_then(|event| event.attributes.iter()
                .find(|attr| attr.key == "reward")
                .map(|attr| attr.value.parse::<i32>().unwrap_or(0)))
            .unwrap_or(0);
        
        println!("  {}: Reward = {}", reward_name, reward);
        
        // Check Q-values after this update
        let q_values: crate::msg::GetQValueResponse = app
            .wrap()
            .query_wasm_smart(&trainer_addr, &QueryMsg::GetQValue { 
                car_id: car_id.clone(), 
                state_hash: state_hash.clone()
            })
            .unwrap();
        
        println!("    Q-values: {:?}", q_values.q_values);
    }
}

#[test]
fn test_training_statistics_tracking() {
    let (mut app, _race_engine_addr, trainer_addr) = setup_test_app();

    let car_id = "test_car".to_string();
    
    println!("\n=== Training Statistics Tracking ===");
    
    // Initial stats
    let initial_stats: crate::msg::GetCarTrainingStatsResponse = app
        .wrap()
        .query_wasm_smart(&trainer_addr, &QueryMsg::GetCarTrainingStats { car_id: car_id.clone() })
        .unwrap();
    
    println!("Initial training updates: {}", initial_stats.training_updates);
    
    // Perform training and track progress
    for round in 1..=5 {
        let training_data = create_training_data_for_track("straight", 1);
        
        for update in training_data {
            let msg = ExecuteMsg::UpdateQValue {
                car_id: update.car_id.clone(),
                state_hash: update.state_hash.clone(),
                action: update.action,
                reward_type: update.reward_type.clone(),
                next_state_hash: update.next_state_hash.clone(),
            };
            
            app.execute_contract(
                Addr::unchecked("owner"),
                trainer_addr.clone(),
                &msg,
                &[],
            ).unwrap();
        }
        
        // Check stats after each round
        let stats: crate::msg::GetCarTrainingStatsResponse = app
            .wrap()
            .query_wasm_smart(&trainer_addr, &QueryMsg::GetCarTrainingStats { car_id: car_id.clone() })
            .unwrap();
        
        println!("After round {}: {} training updates", round, stats.training_updates);
    }
    
    // Final stats
    let final_stats: crate::msg::GetCarTrainingStatsResponse = app
        .wrap()
        .query_wasm_smart(&trainer_addr, &QueryMsg::GetCarTrainingStats { car_id: car_id.clone() })
        .unwrap();
    
    println!("Final training updates: {}", final_stats.training_updates);
    println!("Average updates per round: {:.1}", final_stats.training_updates as f32 / 5.0);
}

#[test]
fn test_error_handling() {
    let (mut app, _race_engine_addr, trainer_addr) = setup_test_app();

    let car_id = "test_car".to_string();
    
    println!("\n=== Error Handling Test ===");
    
    // Test invalid action
    let msg = ExecuteMsg::UpdateQValue {
        car_id: car_id.clone(),
        state_hash: "test_state".to_string(),
        action: 10, // Invalid action (should be 0-4)
        reward_type: RewardType::Distance(10),
        next_state_hash: None,
    };
    
    let result = app.execute_contract(
        Addr::unchecked("owner"),
        trainer_addr.clone(),
        &msg,
        &[],
    );
    
    assert!(result.is_err());
    println!("  Invalid action error handled correctly");
    
    // Test valid training
    let msg = ExecuteMsg::UpdateQValue {
        car_id: car_id.clone(),
        state_hash: "test_state".to_string(),
        action: 0, // Valid action
        reward_type: RewardType::Distance(10),
        next_state_hash: None,
    };
    
    let result = app.execute_contract(
        Addr::unchecked("owner"),
        trainer_addr.clone(),
        &msg,
        &[],
    );
    
    assert!(result.is_ok());
    println!("  Valid training executed successfully");
} 