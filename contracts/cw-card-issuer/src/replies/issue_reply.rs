use abstract_sdk::{AbstractResponse, Transferable, TransferInterface};
use cosmwasm_std::{DepsMut, Env, Reply, Response};
use cw_asset::Asset;
use cw_utils::parse_reply_instantiate_data;

use crate::contract::{AppResult, GiftcardIssuerApp};
use crate::state::{NftReplyInfo, REPLY_INFO, Bribe, BRIBE_MARKET};

pub fn reply_on_issuance(
    deps: DepsMut,
    env: Env,
    app: GiftcardIssuerApp,
    reply: Reply,
) -> AppResult {
    let created = parse_reply_instantiate_data(reply)?;
    let vote_right = deps.api.addr_validate(&created.contract_address)?;

    let NftReplyInfo {
        price,
        collateral,
        party,
    } = REPLY_INFO.load(deps.storage)?;

    REPLY_INFO.remove(deps.storage);

    BRIBE_MARKET.save(deps.storage, &party, &Bribe { price, contract: vote_right.clone() })?;

    Ok(app.custom_tag_response(Response::new(), "issue_reply", vec![("voter_card", vote_right.as_str())]))
}
