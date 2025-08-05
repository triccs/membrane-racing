

// car_nft/src/msg.rs

use cosmwasm_schema::{cw_serde, QueryResponses};
use std::collections::HashMap;

use crate::types::{CarMetadata, QTableEntry};

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Mint {
        owners: Vec<String>,
        metadata: Option<CarMetadata>,
    },
    UpdateOwner {
        car_id: u128,
        owners: Vec<String>,
        is_add: bool,
    },
    UpdateCarMetadata {
        car_id: u128,
        metadata: CarMetadata,
    },
    Transfer {
        car_id: u128,
        to: String,
    }
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Vec<GetCarInfoResponse>)]
    GetCarInfo { 
        car_id: Option<u128>,
        start_after: Option<u128>,
        limit: Option<u32>,
    },
    #[returns(OwnerOfResponse)]
    OwnerOf { car_id: u128 },
    #[returns(NftInfoResponse)]
    NftInfo { car_id: u128 },
    #[returns(GetQResponse)]
    GetQ { car_id: String, state_hash: Option<String> },
    #[returns(AllTokensResponse)]
    AllTokens {},
}

#[cw_serde]
pub struct GetCarInfoResponse {
    pub car_id: String,
    pub owners: Vec<String>,
    pub metadata: Option<CarMetadata>,
}


#[cw_serde]
pub struct OwnerOfResponse {
    pub owners: Vec<String>,
}

#[cw_serde]
pub struct NftInfoResponse {
    pub car_id: String,
    pub owners: Vec<String>,
    pub metadata: Option<CarMetadata>,
}

#[cw_serde]
pub struct AllTokensResponse {
    pub tokens: Vec<String>,
}

#[cw_serde]
pub struct GetQResponse {
    pub car_id: String,
    pub q_values: Vec<QTableEntry>,
}