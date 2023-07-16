use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, StdError, Uint128, VoteOption as CwVoteOption};
use cw_asset::AssetError;
use cw_storage_plus::{Item, Map};
use cw_utils::PaymentError;
use osmosis_std::types::cosmos::gov::v1beta1::VoteOption;
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

    #[error("{0}")]
    Asset(#[from] AssetError),

    #[error("The giftcard doesn't have enough balance to spend {0}")]
    InsufficientBalance(Uint128),

    #[error("Proposal {0} closed")]
    NotVotingPeriod(u64),

    #[error("Proposal {0} not found")]
    ProposalNotFound(u64),

    #[error("Did not vote on {0}")]
    DidNotVote(u64),

    #[error("proposal {0} closed")]
    ProposalClosed(u64),

    #[error("no party vote option")]
    NoPartyVoteOption(u64)
}

#[cw_serde]
pub struct InstantiateMsg {
    pub party: String,
}

#[cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
pub enum ExecuteMsg {
    // Change owner
    Transfer { owner: String },
    // Vote on behalf of the party
    CastVote { proposal_id: u64, vote_option: CwVoteOption },
    // CHeck vote.
    // Anyone can call this, and if the vote did not go as voted, the collateral is sent to the sender
    VerifyVoteOutcome { proposal_id: u64 },
}

// Queries copied from gauge-orchestrator for now (we could use a common crate for this)
/// Queries the gauge requires from the adapter contract in order to function
#[cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
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
    pub party: String,
    pub collateral: Vec<Coin>,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const VOTES: Map<u64, VoteOption> = Map::new("votes");

#[cw_serde]
pub struct Config {
    /// Address of the owner contract
    pub owner: Addr,
    /// Address of the issuer contract this spends from
    pub party: String,
    pub collateral: Vec<Coin>,
}
