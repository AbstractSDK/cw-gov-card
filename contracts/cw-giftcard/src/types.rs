use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, StdError, Uint128};
use cw_storage_plus::Item;
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("Only owner can call this NFT")]
    NotOwner,

    #[error("This contract only sends {0}")]
    InvalidDenom(String),

    #[error("The giftcard doesn't have enough balance to spend {0}")]
    InsufficientBalance(Uint128),
}

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub allowance: Coin,
}

#[cw_serde]
pub enum ExecuteMsg {
    // Change owner
    Transfer { owner: String },
    // Spend value
    Spend { amount: Coin, recipient: String },
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
    /// Address of the owner contract
    pub owner: String,
    /// Address of the issuer contract this spends from
    pub issuer: String,
    /// How much is left to spend
    pub balance: Coin,
}

pub const CONFIG: Item<Config> = Item::new("config");

#[cw_serde]
pub struct Config {
    /// Address of the owner contract
    pub owner: Addr,
    /// Address of the issuer contract this spends from
    pub issuer: Addr,
    /// How much is left to spend
    pub balance: Coin,
}
