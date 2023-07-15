use abstract_core::objects::{AnsAsset, AssetEntry};
use cosmwasm_std::Coin;
use cw_asset::{Asset, AssetInfo};
use cw_storage_plus::Item;

#[cosmwasm_schema::cw_serde]
pub struct Config {
    // Ans asset that's used to issue cards
    pub issue_asset: AssetEntry,
    // denom of the cards it issues
    pub issue_denom: String,
    // code id of giftcard contract
    pub giftcard_id: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const REPLY_INFO: Item<Coin> = Item::new("reply_info");