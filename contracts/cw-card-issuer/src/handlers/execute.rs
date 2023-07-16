use abstract_core::ibc_host::HostAction::App;
use abstract_sdk::{AbstractResponse, AccountingInterface, Execution, TransferInterface};
use abstract_sdk::features::AccountIdentification;
use crate::contract::{AppResult, GiftcardIssuerApp};
use crate::msg::{AppExecuteMsg, GiftCardInstantiateMsg};
use crate::replies::REPLY_ID_INIT;
use crate::state::{CONFIG, BRIBE_MARKET, REPLY_INFO, NftReplyInfo, Bribe};
use cosmwasm_std::{to_binary, Coin, DepsMut, Env, MessageInfo, Response, SubMsg, WasmMsg, ensure_eq, ensure, wasm_execute, CosmosMsg};
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
        AppExecuteMsg::Issue { label, price, collateral } => issue_bribe(deps, env, info, app, price, collateral, label),
        AppExecuteMsg::Bribe { party } => bribe(deps, env, info, app, party),
        // AppExecuteMsg::Spend { amount, recipient } => spend(deps, env, info, app, amount, recipient),
    }
}

/// Buy a bribe for a party
/// Transfers the bribe to the sender
fn bribe(deps: DepsMut, env: Env, info: MessageInfo, app: GiftcardIssuerApp, party: String) -> AppResult {
    let party = deps.api.addr_validate(&party)?;
    let bribe = BRIBE_MARKET.may_load(deps.storage, &party)?.ok_or(AppError::NotIssued(party.to_string()))?;

    // we can buy the bribe for the party if we pay the price
    must_pay(&info, &bribe.price.denom)?;

    ensure_eq!(info.funds[0].amount, bribe.price.amount, AppError::InsufficientFunds {
        provided: info.funds[0].amount,
        required: bribe.price.amount,
    });

    // transfer the bribe to the party
    let transfer_right: CosmosMsg = wasm_execute(bribe.contract, &cw_gov_card::types::ExecuteMsg::Transfer {
        owner: info.sender.to_string(),
    }, vec![])?.into();

    BRIBE_MARKET.remove(deps.storage, &party);

    Ok(app.custom_tag_response(Response::new().add_message(transfer_right), "bribe", vec![("party", party.as_str())]))
}

/// Issue a new right to vote
/// The authz rights are assumed to have been given alreayd (and cannot be checked here)
pub fn issue_bribe(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    app: GiftcardIssuerApp,
    price: Coin,
    collateral: Coin,
    label: Option<String>,
) -> AppResult {
    // check for existing
    ensure!(!BRIBE_MARKET.has(deps.storage, &info.sender), AppError::AlreadyIssued(info.sender.to_string()));

    let cfg = CONFIG.load(deps.storage)?;

    REPLY_INFO.save(deps.storage, &NftReplyInfo {
        party: info.sender.clone(),
        price,
        collateral: collateral.clone(),
    })?;

    let msg = GiftCardInstantiateMsg {
        party: info.sender.into(),
    };

    let msg = WasmMsg::Instantiate {
        admin: None,
        code_id: cfg.giftcard_id,
        msg: to_binary(&msg)?,
        // forward the funds as collateral
        funds: vec![collateral.clone()],
        label: label.unwrap_or_else(|| "Right to Vote".to_string()),
    };
    let msg = SubMsg::reply_on_success(msg, REPLY_ID_INIT);

    // .add_messages(deposit_msgs)
    // store amount for reply
    Ok(app.custom_tag_response(Response::new().add_submessage(msg), "issue", vec![("collateral", collateral.to_string())]))
}

// fn spend(deps: DepsMut, env: Env, info: MessageInfo, app: GiftcardIssuerApp, amount: Coin, recipient: String) -> AppResult {
//     // spend from the account
//     let cfg = CONFIG.load(deps.storage)?;
//     let sender = info.sender;
//     ensure!(RIGHTS_MARKET.has(deps.storage, &sender), AppError::OnlySpendViaAuthz(sender));
//
//     let value = app.bank(deps.as_ref()).balance(&cfg.issue_asset)?;
//
//     ensure!(value.amount >= amount.amount, AppError::InsufficientFunds {
//         balance: value,
//         required: amount,
//     });
//     let deposited = deps.querier.query_balance(app.proxy_address(deps.as_ref())?, "uosmo")?;
//
//     let withdrawal = app.bank(deps.as_ref()).transfer(vec![amount.clone()], &deps.api.addr_validate(&recipient)?)?;
//     let withdrawal = app.executor(deps.as_ref()).execute(vec![withdrawal])?;
//
//     Ok(app.custom_tag_response(Response::new().add_message(withdrawal), "spend", vec![("amount", amount.to_string())]))
// }
