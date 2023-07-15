use abstract_core::objects::{AnsAsset, AssetEntry};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::Coin;
use cw_asset::Asset;
use crate::contract::GiftcardIssuerApp;

// This is used for type safety
// The second part is used to indicate the messages are used as the apps messages
// This is equivalent to
// pub type InstantiateMsg = <PaymentApp as abstract_sdk::base::InstantiateEndpoint>::InstantiateMsg;
// pub type ExecuteMsg = <PaymentApp as abstract_sdk::base::ExecuteEndpoint>::ExecuteMsg;
// pub type QueryMsg = <PaymentApp as abstract_sdk::base::QueryEndpoint>::QueryMsg;
// pub type MigrateMsg = <PaymentApp as abstract_sdk::base::MigrateEndpoint>::MigrateMsg;

// impl app::AppExecuteMsg for AppExecuteMsg {}
// impl app::AppQueryMsg for AppQueryMsg {}
abstract_app::app_messages!(GiftcardIssuerApp, AppExecuteMsg, AppQueryMsg);

/// PaymentApp instantiate message
#[cosmwasm_schema::cw_serde]
pub struct AppInstantiateMsg {
    // denom of the cards it issues
    // (We could allow multiple denoms in the future if desired, but this is simpler for now)
    pub issue_asset: AssetEntry,
    // module id of giftcard contract (TODO
    pub giftcard_id: u64,
}

/// PaymentApp execute messages
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
#[cfg_attr(feature = "interface", impl_into(ExecuteMsg))]
pub enum AppExecuteMsg {
    // Issue a new card
    #[cfg_attr(feature = "interface", payable)]
    Issue { label: Option<String> },
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
