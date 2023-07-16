use abstract_core::objects::{AssetEntry, ContractEntry};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::Coin;

use crate::contract::GiftcardIssuerApp;

abstract_app::app_msg_types!(GiftcardIssuerApp, AppExecuteMsg, AppQueryMsg);

/// PaymentApp instantiate message
#[cosmwasm_schema::cw_serde]
pub struct AppInstantiateMsg {
    // module id of giftcard contract
    pub giftcard_module_id: u64,
    // pub giftcard_module_id: String,
}

/// PaymentApp execute messages
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
#[cfg_attr(feature = "interface", impl_into(ExecuteMsg))]
pub enum AppExecuteMsg {
    // Issue a new card, collateral is sent funds
    #[cfg_attr(feature = "interface", payable)]
    Issue {
        // collateral to be returned when rights are sold
        collateral: Coin,
        // price of the rights listed
        price: Coin,
        // optional label for the card
        label: Option<String>,
    },
    #[cfg_attr(feature = "interface", payable)]
    Bribe {
        // the party whose rights are being sold
        party: String,
    }
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
    // code id of giftcard contract
    pub giftcard_id: u64,
}

// TODO: shared api package, don't copy
#[cosmwasm_schema::cw_serde]
pub struct GiftCardInstantiateMsg {
    pub party: String,
}
