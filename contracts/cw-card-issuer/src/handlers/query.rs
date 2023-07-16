use cosmwasm_std::{to_binary, Binary, Deps, Env, StdResult};

use crate::contract::{AppResult, GiftcardIssuerApp};
use crate::msg::{AppQueryMsg, ConfigResponse};
use crate::state::CONFIG;

pub fn query_handler(
    deps: Deps,
    _env: Env,
    _app: &GiftcardIssuerApp,
    msg: AppQueryMsg,
) -> AppResult<Binary> {
    match msg {
        AppQueryMsg::Config {} => to_binary(&query_config(deps)?),
    }
    .map_err(Into::into)
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let cfg = CONFIG.load(deps.storage)?;

    let cfg = ConfigResponse {
        giftcard_id: cfg.giftcard_id,
    };
    Ok(cfg)
}
