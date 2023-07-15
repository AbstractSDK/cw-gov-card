use crate::contract::{AppResult, GiftcardIssuerApp};
use crate::msg::{AppExecuteMsg, GiftCardInstantiateMsg};
use crate::replies::REPLY_ID_INIT;
use crate::state::{CONFIG, REPLY_INFO};
use cosmwasm_std::{to_binary, Coin, DepsMut, Env, MessageInfo, Response, SubMsg, WasmMsg};
use cw_utils::must_pay;

pub fn execute_handler(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    app: GiftcardIssuerApp,
    msg: AppExecuteMsg,
) -> AppResult {
    match msg {
        AppExecuteMsg::Issue { label } => issue(deps, info, app, label),
    }
}

pub fn issue(
    deps: DepsMut,
    info: MessageInfo,
    _app: GiftcardIssuerApp,
    label: Option<String>,
) -> AppResult {
    let cfg = CONFIG.load(deps.storage)?;
    let amount = must_pay(&info, &cfg.issue_denom)?;

    // instantitate giftcard
    let allowance = Coin {
        amount,
        denom: cfg.issue_denom,
    };

    REPLY_INFO.save(deps.storage, &allowance)?;

    let msg = GiftCardInstantiateMsg {
        owner: info.sender.into(),
        allowance: allowance.clone(),
    };
    let msg = WasmMsg::Instantiate {
        admin: None,
        code_id: cfg.giftcard_id,
        msg: to_binary(&msg)?,
        funds: vec![allowance],
        label: label.unwrap_or_else(|| "Awesome Gift Card".to_string()),
    };
    let msg = SubMsg::reply_on_success(msg, REPLY_ID_INIT);

    // store amount for reply
    Ok(Response::new().add_submessage(msg))
}
