use abstract_core::ibc_host::HostAction::App;
use abstract_sdk::{AbstractResponse, AccountingInterface, Execution, TransferInterface};
use abstract_sdk::features::AccountIdentification;
use crate::contract::{AppResult, GiftcardIssuerApp};
use crate::msg::{AppExecuteMsg, GiftCardInstantiateMsg};
use crate::replies::REPLY_ID_INIT;
use crate::state::{CONFIG, GIFTCARDS, REPLY_INFO};
use cosmwasm_std::{to_binary, Coin, DepsMut, Env, MessageInfo, Response, SubMsg, WasmMsg, ensure_eq, ensure};
use cw_utils::must_pay;
use crate::error::AppError;

pub fn execute_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: GiftcardIssuerApp,
    msg: AppExecuteMsg,
) -> AppResult {
    match msg {
        AppExecuteMsg::Issue { label } => issue(deps, env, info, app, label),
        AppExecuteMsg::Spend { amount, recipient } => spend(deps, env, info, app, amount, recipient),
    }
}

fn spend(deps: DepsMut, env: Env, info: MessageInfo, app: GiftcardIssuerApp, amount: Coin, recipient: String) -> AppResult {
    // spend from the account
    let cfg = CONFIG.load(deps.storage)?;
    let sender = info.sender.to_string();
    ensure!(GIFTCARDS.contains(deps.storage, &sender), AppError::OnlySpendViaAuthz(sender));

    let value = app.bank(deps.as_ref()).balance(&cfg.issue_asset)?;

    ensure!(value.amount >= amount.amount, AppError::InsufficientFunds {
        balance: value,
        required: amount,
    });
    let deposited = deps.querier.query_balance(app.proxy_address(deps.as_ref())?, "uosmo")?;

    let withdrawal = app.bank(deps.as_ref()).transfer(vec![amount.clone()], &deps.api.addr_validate(&recipient)?)?;
    let withdrawal = app.executor(deps.as_ref()).execute(vec![withdrawal])?;

    Ok(app.custom_tag_response(Response::new().add_message(withdrawal), "spend", vec![("amount", amount.to_string())]))
}

pub fn issue(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: GiftcardIssuerApp,
    label: Option<String>,
) -> AppResult {
    let cfg = CONFIG.load(deps.storage)?;
    let amount = must_pay(&info, &cfg.issue_denom)?;

    // instantitate giftcard
    let allowance = Coin {
        amount,
        denom: cfg.issue_denom,
    };

    let deposit_msgs = app.bank(deps.as_ref()).deposit(info.funds.clone())?;

    REPLY_INFO.save(deps.storage, &allowance)?;

    let msg = GiftCardInstantiateMsg {
        owner: info.sender.into(),
        allowance: allowance.clone(),
    };
    let msg = WasmMsg::Instantiate {
        admin: None,
        code_id: cfg.giftcard_id,
        msg: to_binary(&msg)?,
        funds: vec![],
        label: label.unwrap_or_else(|| "Awesome Gift Card".to_string()),
    };
    let msg = SubMsg::reply_on_success(msg, REPLY_ID_INIT);

    // .add_messages(deposit_msgs)
    // store amount for reply
    Ok(app.custom_tag_response(Response::new().add_messages(deposit_msgs).add_submessage(msg), "issue", vec![("amount", amount.to_string())]))
}
