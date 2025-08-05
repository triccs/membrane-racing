use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

use crate::types::{TrackTile, TrackInfo, TileProperties};

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    AddTrack {
        name: String,
        width: u8,
        height: u8,
        layout: Vec<Vec<TileProperties>>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetTrackResponse)]
    GetTrack { track_id: Uint128 },
    #[returns(ListTracksResponse)]
    ListTracks {
        start_after: Option<u128>,
        limit: Option<u32>,
    },
}

#[cw_serde]
pub struct GetTrackResponse {
    pub track_id: u128,
    pub name: String,
    pub width: u8,
    pub height: u8,
    pub layout: Vec<Vec<TrackTile>>,
}

#[cw_serde]
pub struct ListTracksResponse {
    pub tracks: Vec<TrackInfo>,
} 