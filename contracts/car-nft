// car_nft/src/contract.rs

use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo, Response, StdResult, Binary};
use cw2::set_contract_version;
use crate::msg::{InstantiateMsg, ExecuteMsg, QueryMsg};
use crate::state::{CarState, TrackPerformance, QTable, CAR_STATE, Q_TABLE};
use crate::error::ContractError;

const CONTRACT_NAME: &str = "crates.io:car-nft";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}

pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Mint { car_id } => execute_mint(deps, info, car_id),
        ExecuteMsg::Burn { car_id } => execute_burn(deps, info, car_id),
    }
}

fn execute_mint(
    deps: DepsMut,
    info: MessageInfo,
    car_id: u64,
) -> Result<Response, ContractError> {
    let car = CarState {
        owner: info.sender.clone(),
        training_steps: 0,
        total_races: 0,
        history: vec![],
    };
    CAR_STATE.save(deps.storage, car_id, &car)?;
    Ok(Response::new().add_attribute("action", "mint").add_attribute("car_id", car_id.to_string()))
}

fn execute_burn(
    deps: DepsMut,
    info: MessageInfo,
    car_id: u64,
) -> Result<Response, ContractError> {
    let car = CAR_STATE.load(deps.storage, car_id)?;
    if car.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    CAR_STATE.remove(deps.storage, car_id);
    Q_TABLE.remove_prefix(deps.storage, &car_id.to_be_bytes());
    Ok(Response::new().add_attribute("action", "burn").add_attribute("car_id", car_id.to_string()))
}

pub fn query(
    deps: Deps,
    _env: Env,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetQ { car_id, state_hash } => to_binary(&query_q_values(deps, car_id, state_hash)?),
        QueryMsg::Metadata { car_id } => to_binary(&CAR_STATE.load(deps.storage, car_id)?),
    }
}

fn query_q_values(
    deps: Deps,
    car_id: u64,
    state_hash: String,
) -> StdResult<[i32; 5]> {
    let q_values = Q_TABLE.may_load(deps.storage, (car_id, state_hash))?.unwrap_or([0; 5]);
    Ok(q_values)
} 

// car_nft/src/state.rs

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

pub type QTable = Map<(u64, String), [i32; 5]>; // car_id + state_hash -> action values
pub static Q_TABLE: QTable = Map::new("qtable");

pub static CAR_STATE: Map<u64, CarState> = Map::new("car_state");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct CarState {
    pub owner: Addr,
    pub training_steps: u64,
    pub total_races: u64,
    pub history: Vec<(u64, TrackPerformance)>, // track_id -> performance
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct TrackPerformance {
    pub wins: u64,
    pub losses: u64,
    pub best_time: u64,
}

// car_nft/src/msg.rs

use cosmwasm_std::Binary;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum ExecuteMsg {
    Mint { car_id: u64 },
    Burn { car_id: u64 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum QueryMsg {
    GetQ { car_id: u64, state_hash: String },
    Metadata { car_id: u64 },
}

// car_nft/src/error.rs

use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("Unauthorized")]
    Unauthorized {},

    #[error("{0}")]
    Std(#[from] StdError),
} 
