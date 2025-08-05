// car_nft/src/contract.rs

use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw_storage_plus::Bound;
use racing::car::GetCarInfoResponse;

use crate::error::CarError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{get_car_info, set_car_info, CarInfo, ADMIN, CAR_ID_COUNTER, CAR_INFO};
use racing::types::{QTableEntry, CarMetadata};

const MAX_LIMIT: u32 = 32;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, CarError> {
    let admin = deps.api.addr_validate(&msg.admin)?;
    ADMIN.save(deps.storage, &admin)?;
    
    // Initialize car ID counter to 0
    CAR_ID_COUNTER.save(deps.storage, &Uint128::zero())?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("admin", admin))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, CarError> {
    match msg {
        ExecuteMsg::Mint {
            owners,
            metadata,
        } => execute_mint(deps, env, info, owners, metadata),
        ExecuteMsg::UpdateOwner { car_id, owners, is_add } => {
            execute_update_owner(deps, info, car_id, owners, is_add)
        }
        ExecuteMsg::UpdateCarMetadata { car_id, metadata } => {
            execute_update_car_metadata(deps, info, car_id, metadata)
        }
        ExecuteMsg::Transfer { car_id, to } => execute_transfer(deps, info, car_id, to),
    }
}

pub fn execute_mint(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    owners: Vec<String>,
    metadata: Option<CarMetadata>,
) -> Result<Response, CarError> {
    //Load Car ID 
    let car_id = CAR_ID_COUNTER.load(deps.storage)?;
    CAR_ID_COUNTER.save(deps.storage, &(car_id + Uint128::one()))?;


    // Validate all owner addresses
    let owner_addrs: Result<Vec<_>, _> = owners
        .iter()
        .map(|owner| deps.api.addr_validate(owner))
        .collect();
    let owner_addrs = owner_addrs?;
    
    let car_info = crate::state::CarInfo {
        owners: owner_addrs,
        metadata,
        created_at: env.block.time.seconds(),
    };

    set_car_info(deps.storage, car_id.u128(), car_info)?;

    Ok(Response::new()
        .add_attribute("method", "mint")
        .add_attribute("car_id", car_id.to_string())
        .add_attribute("owners", owners.join(",")))
}

pub fn execute_update_owner(
    deps: DepsMut,
    info: MessageInfo,
    car_id: u128,
    new_owners: Vec<String>,
    is_add: bool,
) -> Result<Response, CarError> {
    let mut car_info = get_car_info(deps.storage, car_id)?;
    
    // Check if sender is one of the current owners
    if !car_info.owners.contains(&info.sender) {
        return Err(CarError::Unauthorized {});
    }

    let new_owner_addrs: Result<Vec<_>, _> = new_owners
        .iter()
        .map(|owner| deps.api.addr_validate(owner))
        .collect();
    let new_owner_addrs = new_owner_addrs?;

    if is_add {
        // Add new owners (avoid duplicates)
        for owner in new_owner_addrs {
            if !car_info.owners.contains(&owner) {
                car_info.owners.push(owner);
            }
        }
    } else {
        // Remove owners
        car_info.owners = car_info.owners.into_iter().filter(|owner| !new_owner_addrs.contains(owner)).collect();
        if car_info.owners.is_empty() {
            return Err(CarError::CarHasNoOwners { car_id });
        }
    }

    set_car_info(deps.storage, car_id, car_info)?;

    Ok(Response::new()
        .add_attribute("method", "update_owner")
        .add_attribute("car_id", car_id.to_string())
        .add_attribute("is_add", is_add.to_string()))
}

pub fn execute_update_car_metadata(
    deps: DepsMut,
    info: MessageInfo,
    car_id: u128,
    metadata: CarMetadata,
) -> Result<Response, CarError> {
    let car_info = get_car_info(deps.storage, car_id)?;
    
    // Check if sender is one of the owners
    if !car_info.owners.contains(&info.sender) {
        return Err(CarError::Unauthorized {});
    }

    let mut updated_car_info = car_info;
    updated_car_info.metadata = Some(metadata);
    set_car_info(deps.storage, car_id, updated_car_info)?;

    Ok(Response::new()
        .add_attribute("method", "update_car_metadata")
        .add_attribute("car_id", car_id.to_string()))
}

pub fn execute_transfer(
    deps: DepsMut,
    info: MessageInfo,
    car_id: u128,
    to: String,
) -> Result<Response, CarError> {
    let car_info = get_car_info(deps.storage, car_id)?;
    
    // Check if sender is one of the owners
    if !car_info.owners.contains(&info.sender) {
        return Err(CarError::Unauthorized {});
    }

    let new_owner = deps.api.addr_validate(&to)?;
    let mut updated_car_info = car_info;
    updated_car_info.owners = vec![new_owner.clone()];
    set_car_info(deps.storage, car_id, updated_car_info)?;

    Ok(Response::new()
        .add_attribute("method", "transfer")
        .add_attribute("car_id", car_id.to_string())
        .add_attribute("from", info.sender)
        .add_attribute("to", new_owner))
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCarInfo { car_id, start_after, limit } => to_json_binary(&query_car_info(deps, car_id, start_after, limit).map_err(|e| cosmwasm_std::StdError::generic_err(e.to_string()))?),
        QueryMsg::OwnerOf { car_id } => to_json_binary(&query_owner_of(deps, car_id).map_err(|e| cosmwasm_std::StdError::generic_err(e.to_string()))?),
        QueryMsg::NftInfo { car_id } => to_json_binary(&query_nft_info(deps, car_id).map_err(|e| cosmwasm_std::StdError::generic_err(e.to_string()))?),
        QueryMsg::GetQ { car_id, state_hash } => to_json_binary(&query_q_values(deps, car_id, state_hash).map_err(|e| cosmwasm_std::StdError::generic_err(e.to_string()))?),
        QueryMsg::AllTokens {} => to_json_binary(&query_all_tokens(deps).map_err(|e| cosmwasm_std::StdError::generic_err(e.to_string()))?),
    }
}

pub fn query_car_info(deps: Deps, car_id: Option<u128>, start_after: Option<u128>, limit: Option<u32>) -> Result<Vec<GetCarInfoResponse>, CarError> {
    if let Some(car_id) = car_id {
        let car_info = get_car_info(deps.storage, car_id)?;
        
        Ok(vec![GetCarInfoResponse {
            car_id: car_id.to_string(),
            owners: car_info.owners.iter().map(|addr| addr.to_string()).collect(),
            metadata: car_info.metadata,
        }])
    } else {
        let mut cars = vec![];
        
        //Get limit
        let limit = limit.unwrap_or(MAX_LIMIT) as usize;

        //Get start
        let start = if let Some(start) = start_after {
            Some(Bound::exclusive(start))
        } else {
            None
        };
        let range = CAR_INFO
            .range(deps.storage, start, None, cosmwasm_std::Order::Ascending)
            .take(limit)
            .collect::<Result<Vec<(u128, CarInfo)>, _>>()?;
        for item in range {
            let (car_id, car_info) = item;
            cars.push(GetCarInfoResponse {
                car_id: car_id.to_string(),
                owners: car_info.owners.iter().map(|addr| addr.to_string()).collect(),
                metadata: car_info.metadata,
            });
        }

        Ok(cars)
    }
}

pub fn query_owner_of(deps: Deps, car_id: u128) -> Result<crate::msg::OwnerOfResponse, CarError> {
    let car_info = get_car_info(deps.storage, car_id)?;
    
    Ok(crate::msg::OwnerOfResponse {
        owners: car_info.owners.iter().map(|addr| addr.to_string()).collect(),
    })
}

pub fn query_nft_info(deps: Deps, car_id: u128) -> Result<crate::msg::NftInfoResponse, CarError> {
    let car_info = get_car_info(deps.storage, car_id)?;
    
    Ok(crate::msg::NftInfoResponse {
        car_id: car_id.to_string(),
        owners: car_info.owners.iter().map(|addr| addr.to_string()).collect(),
        metadata: car_info.metadata,
    })
}

pub fn query_q_values(deps: Deps, car_id: String, state_hash: Option<String>) -> Result<crate::msg::GetQResponse, CarError> {
    let car_id_u128 = car_id.parse::<u128>().map_err(|_| CarError::InvalidCarId { car_id: car_id.clone() })?;
    
    if let Some(state_hash) = state_hash {
        // Query specific state
        let q_values = crate::state::get_q_values(deps.storage, car_id_u128, &state_hash)
            .unwrap_or([0, 0, 0, 0]);
        
        Ok(crate::msg::GetQResponse {
            car_id,
            q_values: vec![racing::types::QTableEntry {
                state_hash,
                action_values: q_values,
            }],
        })
    } else {
        // Query all states for this car
        let q_values = crate::state::get_all_q_values_for_car(deps.storage, car_id_u128)?;
        
        Ok(crate::msg::GetQResponse {
            car_id,
            q_values,
        })
    }
}

pub fn query_all_tokens(deps: Deps) -> Result<crate::msg::AllTokensResponse, CarError> {
    let mut tokens = vec![];
    let range = CAR_INFO.range(deps.storage, None, None, cosmwasm_std::Order::Ascending);
    for item in range {
        let (car_id, _) = item?;
        tokens.push(car_id.to_string());
    }
    
    Ok(crate::msg::AllTokensResponse { tokens })
}

