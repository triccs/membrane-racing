use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TrainerError {
    #[error("Invalid action: {action}")]
    InvalidAction { action: usize },

    #[error("Training session already active for car: {car_id}")]
    TrainingSessionAlreadyActive { car_id: String },

    #[error("Training session not found for car: {car_id}")]
    TrainingSessionNotFound { car_id: String },

    #[error("Invalid training configuration")]
    InvalidTrainingConfig,

    #[error("Invalid reward configuration")]
    InvalidRewardConfig,

    #[error("Storage error: {0}")]
    Storage(#[from] cosmwasm_std::StdError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json_wasm::ser::Error),

    #[error("Deserialization error: {0}")]
    Deserialization(#[from] serde_json_wasm::de::Error),
}

pub type TrainerResult<T> = Result<T, TrainerError>; 