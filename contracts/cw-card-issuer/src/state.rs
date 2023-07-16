use abstract_core::objects::AssetEntry;
use cosmwasm_std::{Addr, Coin};
use cw_item_set::Set;

use cw_storage_plus::{Item, Map};

#[cosmwasm_schema::cw_serde]
pub struct Config {
    // module id of giftcard contract
    pub giftcard_module_id: String,
    // code id of giftcard contract
    pub giftcard_id: u64,
}

#[cosmwasm_schema::cw_serde]
pub struct NftReplyInfo {
    pub party: Addr,
    pub price: Coin,
    pub collateral: Coin,
}

#[cosmwasm_schema::cw_serde]
pub struct Bribe {
    pub contract: Addr,
    pub price: Coin,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const REPLY_INFO: Item<NftReplyInfo> = Item::new("reply_info");
pub const BRIBE_MARKET: Map<&Addr, Bribe> = Map::new("rights");
