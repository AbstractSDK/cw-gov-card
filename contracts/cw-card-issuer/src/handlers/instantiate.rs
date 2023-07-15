use abstract_sdk::features::AbstractNameService;
use abstract_sdk::AbstractResponse;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw_asset::AssetInfo;

use crate::contract::{AppResult, GiftcardIssuerApp};
use crate::error::AppError;
use crate::msg::AppInstantiateMsg;
use crate::state::{Config, CONFIG};

pub fn instantiate_handler(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    app: GiftcardIssuerApp,
    msg: AppInstantiateMsg,
) -> AppResult {
    let ans = app.name_service(deps.as_ref());

    let issue_asset_info = ans.query(&msg.issue_asset)?;

    let issue_denom = match issue_asset_info {
        AssetInfo::Native(denom) => Ok(denom),
        _ => Err(AppError::OnlyNativeSupported),
    }?;

    let cfg = Config {
        issue_asset: msg.issue_asset,
        issue_denom,
        giftcard_id: msg.giftcard_id,
    };

    CONFIG.save(deps.storage, &cfg)?;

    Ok(app.tag_response(Response::new(), "instantiate"))
}
