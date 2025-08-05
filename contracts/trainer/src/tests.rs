use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coins, from_json};

use crate::contract::{execute, instantiate, query};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("creator", &coins(1000, "earth"));

    let msg = InstantiateMsg {
        admin: "creator".to_string(),
    };

    let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(0, res.messages.len());
}

#[test]
fn test_update_q_value() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("creator", &coins(1000, "earth"));

    // Instantiate
    let msg = InstantiateMsg {
        admin: "creator".to_string(),
    };
    instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Update Q-value
    let msg = ExecuteMsg::UpdateQValue {
        car_id: "car_1".to_string(),
        state_hash: "state_1".to_string(),
        action: 2,
        reward_type: racing::types::RewardType::Distance(10),
        next_state_hash: Some("state_2".to_string()),
    };

    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(0, res.messages.len());
    
    // Verify Q-value was updated
    let query_msg = QueryMsg::GetCarTrainingStats { car_id: "car_1".to_string() };
    let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
    let stats_response: crate::msg::GetCarTrainingStatsResponse = from_json(&res).unwrap();
    
    assert_eq!(stats_response.training_updates, 1);
}

#[test]
fn test_batch_update_q_values() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("creator", &coins(1000, "earth"));

    // Instantiate
    let msg = InstantiateMsg {
        admin: "creator".to_string(),
    };
    instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Batch update Q-values
    let updates = vec![
        racing::types::QUpdate {
            car_id: "car_1".to_string(),
            state_hash: "state_1".to_string(),
            action: 0,
            reward_type: racing::types::RewardType::Distance(10),
            next_state_hash: Some("state_2".to_string()),
        },
        racing::types::QUpdate {
            car_id: "car_1".to_string(),
            state_hash: "state_2".to_string(),
            action: 1,
            reward_type: racing::types::RewardType::Rank(1),
            next_state_hash: None,
        },
    ];

    let msg = ExecuteMsg::BatchUpdateQValues { updates };

    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(0, res.messages.len());
    
    // Verify Q-values were updated
    let query_msg = QueryMsg::GetCarTrainingStats { car_id: "car_1".to_string() };
    let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
    let stats_response: crate::msg::GetCarTrainingStatsResponse = from_json(&res).unwrap();
    
    assert_eq!(stats_response.training_updates, 2);
}

#[test]
fn test_record_track_result() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("creator", &coins(1000, "earth"));

    // Instantiate
    let msg = InstantiateMsg {
        admin: "creator".to_string(),
    };
    instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Record track result
    let msg = ExecuteMsg::RecordTrackResult {
        car_id: "car_1".to_string(),
        track_id: "track_1".to_string(),
        won: true,
        steps_taken: 15,
    };

    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(0, res.messages.len());
    
    // Verify track result was recorded
    let query_msg = QueryMsg::GetTrackResults {
        car_id: "car_1".to_string(),
        track_id: "track_1".to_string(),
    };
    let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
    let results_response: crate::msg::GetTrackResultsResponse = from_json(&res).unwrap();
    
    assert_eq!(results_response.wins, 1);
    assert_eq!(results_response.losses, 0);
}

#[test]
fn test_multiple_track_results() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("creator", &coins(1000, "earth"));

    // Instantiate
    let msg = InstantiateMsg {
        admin: "creator".to_string(),
    };
    instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Record multiple track results
    for i in 0..3 {
        let won = i % 2 == 0; // Alternate wins and losses
        let msg = ExecuteMsg::RecordTrackResult {
            car_id: "car_1".to_string(),
            track_id: "track_1".to_string(),
            won,
            steps_taken: 10 + i as u32,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    }
    
    // Verify track results
    let query_msg = QueryMsg::GetTrackResults {
        car_id: "car_1".to_string(),
        track_id: "track_1".to_string(),
    };
    let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
    let results_response: crate::msg::GetTrackResultsResponse = from_json(&res).unwrap();
    
    assert_eq!(results_response.wins, 2);
    assert_eq!(results_response.losses, 1);
}

#[test]
fn test_different_reward_types() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("creator", &coins(1000, "earth"));

    // Instantiate
    let msg = InstantiateMsg {
        admin: "creator".to_string(),
    };
    instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Test different reward types
    let reward_types = vec![
        racing::types::RewardType::Distance(10),
        racing::types::RewardType::Stuck,
        racing::types::RewardType::Wall,
        racing::types::RewardType::NoMove,
        racing::types::RewardType::Explore,
        racing::types::RewardType::Rank(1),
    ];

    for (i, reward_type) in reward_types.iter().enumerate() {
        let msg = ExecuteMsg::UpdateQValue {
            car_id: "car_1".to_string(),
            state_hash: format!("state_{}", i),
            action: (i % 5) as u8, // Ensure action is 0-4
            reward_type: reward_type.clone(),
            next_state_hash: Some(format!("state_{}", i + 1)),
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    }
    
    // Verify all updates were recorded
    let query_msg = QueryMsg::GetCarTrainingStats { car_id: "car_1".to_string() };
    let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
    let stats_response: crate::msg::GetCarTrainingStatsResponse = from_json(&res).unwrap();
    
    assert_eq!(stats_response.training_updates, 6);
}

#[test]
fn test_multiple_cars_training() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("creator", &coins(1000, "earth"));

    // Instantiate
    let msg = InstantiateMsg {
        admin: "creator".to_string(),
    };
    instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Train multiple cars
    for car_id in 1..=3 {
        let msg = ExecuteMsg::UpdateQValue {
            car_id: format!("car_{}", car_id),
            state_hash: "state_1".to_string(),
            action: 0,
            reward_type: racing::types::RewardType::Distance(10),
            next_state_hash: Some("state_2".to_string()),
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    }
    
    // Verify each car has training stats
    for car_id in 1..=3 {
        let query_msg = QueryMsg::GetCarTrainingStats { 
            car_id: format!("car_{}", car_id) 
        };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let stats_response: crate::msg::GetCarTrainingStatsResponse = from_json(&res).unwrap();
        
        assert_eq!(stats_response.training_updates, 1);
    }
}

#[test]
fn test_track_results_for_multiple_cars() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("creator", &coins(1000, "earth"));

    // Instantiate
    let msg = InstantiateMsg {
        admin: "creator".to_string(),
    };
    instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Record track results for multiple cars
    for car_id in 1..=3 {
        let msg = ExecuteMsg::RecordTrackResult {
            car_id: format!("car_{}", car_id),
            track_id: "track_1".to_string(),
            won: car_id % 2 == 1, // Odd cars win
            steps_taken: 10 + car_id as u32,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    }
    
    // Verify track results for each car
    for car_id in 1..=3 {
        let query_msg = QueryMsg::GetTrackResults {
            car_id: format!("car_{}", car_id),
            track_id: "track_1".to_string(),
        };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let results_response: crate::msg::GetTrackResultsResponse = from_json(&res).unwrap();
        
        if car_id % 2 == 1 {
            assert_eq!(results_response.wins, 1);
            assert_eq!(results_response.losses, 0);
        } else {
            assert_eq!(results_response.wins, 0);
            assert_eq!(results_response.losses, 1);
        }
    }
}

// Integration tests using cw-multi-test
#[cfg(test)]
mod integration_tests {
    use super::*;
    use cosmwasm_std::Addr;
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    fn trainer_contract() -> Box<dyn Contract<cosmwasm_std::Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        Box::new(contract)
    }

    #[test]
    fn test_integration_q_learning_training() {
        let mut app = AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked("trainer"), coins(1000, "earth"))
                .unwrap();
        });

        // Upload and instantiate trainer contract
        let trainer_contract_id = app.store_code(trainer_contract());
        let trainer_addr = app
            .instantiate_contract(
                trainer_contract_id,
                Addr::unchecked("admin"),
                &InstantiateMsg { admin: "admin".to_string() },
                &[],
                "Trainer",
                None,
            )
            .unwrap();

        // Update Q-value
        let update_msg = ExecuteMsg::UpdateQValue {
            car_id: "car_1".to_string(),
            state_hash: "state_1".to_string(),
            action: 2,
            reward_type: racing::types::RewardType::Distance(10),
            next_state_hash: Some("state_2".to_string()),
        };

        let result = app
            .execute_contract(
                Addr::unchecked("trainer"),
                trainer_addr.clone(),
                &update_msg,
                &[],
            )
            .unwrap();

        // Verify Q-value update was successful
        assert!(result.events.iter().any(|event| {
            event.ty == "wasm" && event.attributes.iter().any(|attr| {
                attr.key == "method" && attr.value == "update_q_value"
            })
        }));

        // Query training stats
        let stats: crate::msg::GetCarTrainingStatsResponse = app
            .wrap()
            .query_wasm_smart(
                &trainer_addr,
                &QueryMsg::GetCarTrainingStats { car_id: "car_1".to_string() }
            )
            .unwrap();

        assert_eq!(stats.training_updates, 1);
    }

    #[test]
    fn test_integration_batch_training() {
        let mut app = AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked("trainer"), coins(1000, "earth"))
                .unwrap();
        });

        // Upload and instantiate trainer contract
        let trainer_contract_id = app.store_code(trainer_contract());
        let trainer_addr = app
            .instantiate_contract(
                trainer_contract_id,
                Addr::unchecked("admin"),
                &InstantiateMsg { admin: "admin".to_string() },
                &[],
                "Trainer",
                None,
            )
            .unwrap();

        // Batch update Q-values
        let updates = vec![
            racing::types::QUpdate {
                car_id: "car_1".to_string(),
                state_hash: "state_1".to_string(),
                action: 0,
                reward_type: racing::types::RewardType::Distance(10),
                next_state_hash: Some("state_2".to_string()),
            },
            racing::types::QUpdate {
                car_id: "car_1".to_string(),
                state_hash: "state_2".to_string(),
                action: 1,
                reward_type: racing::types::RewardType::Rank(1),
                next_state_hash: None,
            },
            racing::types::QUpdate {
                car_id: "car_2".to_string(),
                state_hash: "state_1".to_string(),
                action: 2,
                reward_type: racing::types::RewardType::Stuck,
                next_state_hash: Some("state_3".to_string()),
            },
        ];

        let batch_msg = ExecuteMsg::BatchUpdateQValues { updates };

        let result = app
            .execute_contract(
                Addr::unchecked("trainer"),
                trainer_addr.clone(),
                &batch_msg,
                &[],
            )
            .unwrap();

        // Verify batch update was successful
        assert!(result.events.iter().any(|event| {
            event.ty == "wasm" && event.attributes.iter().any(|attr| {
                attr.key == "method" && attr.value == "batch_update_q_values"
            })
        }));

        // Query training stats for both cars
        let stats1: crate::msg::GetCarTrainingStatsResponse = app
            .wrap()
            .query_wasm_smart(
                &trainer_addr,
                &QueryMsg::GetCarTrainingStats { car_id: "car_1".to_string() }
            )
            .unwrap();

        let stats2: crate::msg::GetCarTrainingStatsResponse = app
            .wrap()
            .query_wasm_smart(
                &trainer_addr,
                &QueryMsg::GetCarTrainingStats { car_id: "car_2".to_string() }
            )
            .unwrap();

        assert_eq!(stats1.training_updates, 2);
        assert_eq!(stats2.training_updates, 1);
    }

    #[test]
    fn test_integration_track_results() {
        let mut app = AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked("trainer"), coins(1000, "earth"))
                .unwrap();
        });

        // Upload and instantiate trainer contract
        let trainer_contract_id = app.store_code(trainer_contract());
        let trainer_addr = app
            .instantiate_contract(
                trainer_contract_id,
                Addr::unchecked("admin"),
                &InstantiateMsg { admin: "admin".to_string() },
                &[],
                "Trainer",
                None,
            )
            .unwrap();

        // Record track results
        for i in 1..=5 {
            let won = i % 2 == 1; // Alternate wins and losses
            let record_msg = ExecuteMsg::RecordTrackResult {
                car_id: "car_1".to_string(),
                track_id: "track_1".to_string(),
                won,
                steps_taken: 10 + i as u32,
            };

            app.execute_contract(
                Addr::unchecked("trainer"),
                trainer_addr.clone(),
                &record_msg,
                &[],
            )
            .unwrap();
        }

        // Query track results
        let results: crate::msg::GetTrackResultsResponse = app
            .wrap()
            .query_wasm_smart(
                &trainer_addr,
                &QueryMsg::GetTrackResults {
                    car_id: "car_1".to_string(),
                    track_id: "track_1".to_string(),
                }
            )
            .unwrap();

        assert_eq!(results.wins, 3);
        assert_eq!(results.losses, 2);
    }

    #[test]
    fn test_integration_multiple_cars_and_tracks() {
        let mut app = AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked("trainer"), coins(1000, "earth"))
                .unwrap();
        });

        // Upload and instantiate trainer contract
        let trainer_contract_id = app.store_code(trainer_contract());
        let trainer_addr = app
            .instantiate_contract(
                trainer_contract_id,
                Addr::unchecked("admin"),
                &InstantiateMsg { admin: "admin".to_string() },
                &[],
                "Trainer",
                None,
            )
            .unwrap();

        // Train multiple cars on multiple tracks
        for car_id in 1..=3 {
            for track_id in 1..=2 {
                // Update Q-values
                let update_msg = ExecuteMsg::UpdateQValue {
                    car_id: format!("car_{}", car_id),
                    state_hash: format!("state_{}", track_id),
                    action: (car_id + track_id) as u8 % 5,
                    reward_type: racing::types::RewardType::Distance(10),
                    next_state_hash: Some(format!("state_{}", track_id + 1)),
                };

                app.execute_contract(
                    Addr::unchecked("trainer"),
                    trainer_addr.clone(),
                    &update_msg,
                    &[],
                )
                .unwrap();

                // Record track results
                let record_msg = ExecuteMsg::RecordTrackResult {
                    car_id: format!("car_{}", car_id),
                    track_id: format!("track_{}", track_id),
                    won: (car_id + track_id) % 2 == 0,
                    steps_taken: 10 + car_id as u32 + track_id as u32,
                };

                app.execute_contract(
                    Addr::unchecked("trainer"),
                    trainer_addr.clone(),
                    &record_msg,
                    &[],
                )
                .unwrap();
            }
        }

        // Verify training stats for each car
        for car_id in 1..=3 {
            let stats: crate::msg::GetCarTrainingStatsResponse = app
                .wrap()
                .query_wasm_smart(
                    &trainer_addr,
                    &QueryMsg::GetCarTrainingStats { car_id: format!("car_{}", car_id) }
                )
                .unwrap();

            assert_eq!(stats.training_updates, 2); // 2 tracks per car
        }

        // Verify track results for each car-track combination
        for car_id in 1..=3 {
            for track_id in 1..=2 {
                let results: crate::msg::GetTrackResultsResponse = app
                    .wrap()
                    .query_wasm_smart(
                        &trainer_addr,
                        &QueryMsg::GetTrackResults {
                            car_id: format!("car_{}", car_id),
                            track_id: format!("track_{}", track_id),
                        }
                    )
                    .unwrap();

                let expected_wins = if (car_id + track_id) % 2 == 0 { 1 } else { 0 };
                let expected_losses = if (car_id + track_id) % 2 == 0 { 0 } else { 1 };

                assert_eq!(results.wins, expected_wins);
                assert_eq!(results.losses, expected_losses);
            }
        }
    }

    #[test]
    fn test_integration_error_handling() {
        let mut app = AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked("trainer"), coins(1000, "earth"))
                .unwrap();
        });

        // Upload and instantiate trainer contract
        let trainer_contract_id = app.store_code(trainer_contract());
        let trainer_addr = app
            .instantiate_contract(
                trainer_contract_id,
                Addr::unchecked("admin"),
                &InstantiateMsg { admin: "admin".to_string() },
                &[],
                "Trainer",
                None,
            )
            .unwrap();

        // First, test a valid update to ensure the contract works
        let valid_update_msg = ExecuteMsg::UpdateQValue {
            car_id: "car_1".to_string(),
            state_hash: "state_1".to_string(),
            action: 2, // Valid action (0-4)
            reward_type: racing::types::RewardType::Distance(10),
            next_state_hash: Some("state_2".to_string()),
        };

        let result = app.execute_contract(
            Addr::unchecked("trainer"),
            trainer_addr.clone(),
            &valid_update_msg,
            &[],
        );

        assert!(result.is_ok()); // Should succeed

        // Now test with invalid action
        let invalid_update_msg = ExecuteMsg::UpdateQValue {
            car_id: "car_1".to_string(),
            state_hash: "state_1".to_string(),
            action: 10, // Invalid action (should be 0-4)
            reward_type: racing::types::RewardType::Distance(10),
            next_state_hash: Some("state_2".to_string()),
        };

        let result = app.execute_contract(
            Addr::unchecked("trainer"),
            trainer_addr.clone(),
            &invalid_update_msg,
            &[],
        );

        // The error is being thrown correctly, but cw-multi-test might handle it differently
        // Let's check if the result contains an error message about invalid action
        match result {
            Ok(_) => panic!("Expected error for invalid action, but got success"),
            Err(e) => {
                let error_str = format!("{:?}", e);
                assert!(error_str.contains("Invalid action") || error_str.contains("Invalid action index"));
            }
        }

        // Try to query non-existent car stats
        let result = app.wrap().query_wasm_smart::<crate::msg::GetCarTrainingStatsResponse>(
            &trainer_addr,
            &QueryMsg::GetCarTrainingStats { car_id: "non_existent".to_string() }
        );

        assert!(result.is_err()); // Should fail because car doesn't exist
    }
} 