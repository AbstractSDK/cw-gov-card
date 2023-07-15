use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, StdError};
use cw_storage_plus::Item;
use cw_utils::{ParseReplyError, PaymentError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("{0}")]
    Parse(#[from] ParseReplyError),

    #[error("Invalid ReplyId: {0}")]
    InvalidReplyId(u64),
}

#[cw_serde]
pub struct InstantiateMsg {
    // denom of the cards it issues
    // (We could allow multiple denoms in the future if desired, but this is simpler for now)
    pub denom: String,
    // code id of giftcard contract
    pub giftcard_id: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    // Issue a new card
    Issue { label: Option<String> },
}

// Queries copied from gauge-orchestrator for now (we could use a common crate for this)
/// Queries the gauge requires from the adapter contract in order to function
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
}

#[cw_serde]
pub struct ConfigResponse {
    // denom of the cards it issues
    pub denom: String,
    // code id of giftcard contract
    pub giftcard_id: u64,
}

#[cw_serde]
pub struct Config {
    // denom of the cards it issues
    pub denom: String,
    // code id of giftcard contract
    pub giftcard_id: u64,
}
