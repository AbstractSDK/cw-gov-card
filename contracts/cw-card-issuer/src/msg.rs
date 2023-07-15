use abstract_core::objects::{AssetEntry, ContractEntry};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::Coin;

use crate::contract::GiftcardIssuerApp;

abstract_app::app_msg_types!(GiftcardIssuerApp, AppExecuteMsg, AppQueryMsg);

/// PaymentApp instantiate message
#[cosmwasm_schema::cw_serde]
pub struct AppInstantiateMsg {
    // denom of the cards it issues
    // (We could allow multiple denoms in the future if desired, but this is simpler for now)
    pub issue_asset: AssetEntry,
    // module id of giftcard contract
    pub giftcard_module_id: u64,
    // pub giftcard_module_id: String,
}

/// PaymentApp execute messages
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
#[cfg_attr(feature = "interface", impl_into(ExecuteMsg))]
pub enum AppExecuteMsg {
    // Issue a new card
    #[cfg_attr(feature = "interface", payable)]
    Issue {
        label: Option<String>,
    },

    // Spend from a card
    Spend {
        amount: Coin,
        recipient: String,
    },
}

#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
#[cfg_attr(feature = "interface", impl_into(QueryMsg))]
#[derive(QueryResponses)]
pub enum AppQueryMsg {
    #[returns(ConfigResponse)]
    Config {},
}

#[cosmwasm_schema::cw_serde]
pub enum AppMigrateMsg {}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    // Ans asset that's used to issue cards
    pub issue_asset: AssetEntry,
    // denom of the cards it issues
    pub issue_denom: String,
    // code id of giftcard contract
    pub giftcard_id: u64,
}

// TODO: shared api package, don't copy
#[cosmwasm_schema::cw_serde]
pub struct GiftCardInstantiateMsg {
    pub owner: String,
    pub allowance: Coin,
}
