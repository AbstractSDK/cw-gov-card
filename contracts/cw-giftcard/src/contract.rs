#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    ensure_eq, to_binary, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult,
};
use cw2::set_contract_version;
use cw_utils::{nonpayable, one_coin};
use osmosis_std::shim::Any;
use osmosis_std::types::cosmos::authz::v1beta1::MsgExec;
use osmosis_std::types::cosmos::bank::v1beta1::MsgSend;
use osmosis_std::types::cosmwasm::wasm::v1::MsgExecuteContract;

use crate::types::{
    Config, ConfigResponse, ContractError, ExecuteMsg, InstantiateMsg, QueryMsg, CONFIG,
};

// version info for migration info
pub const CONTRACT_NAME: &str = "crates.io:cw-giftcard";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(feature = "export", entry_point)]
#[cfg_attr(feature = "interface", cw_orch::interface_entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = deps.api.addr_validate(&msg.owner)?;
    let issuer = info.sender;

    let cfg = Config {
        owner,
        issuer,
        balance: msg.allowance
    };
    CONFIG.save(deps.storage, &cfg)?;
    Ok(Response::new())
}

#[cfg_attr(feature = "export", entry_point)]
#[cfg_attr(feature = "interface", cw_orch::interface_entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Transfer { owner } => transfer(deps, info, owner),
        ExecuteMsg::Redeem { amount, recipient } => redeem(deps, env, info, recipient, amount),
    }
}

pub fn transfer(
    deps: DepsMut,
    info: MessageInfo,
    owner: String,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    let mut cfg = CONFIG.load(deps.storage)?;
    ensure_eq!(info.sender, cfg.owner, ContractError::NotOwner);
    cfg.owner = deps.api.addr_validate(&owner)?;
    CONFIG.save(deps.storage, &cfg)?;
    Ok(Response::new())
}

pub fn redeem(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Coin,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    let mut cfg = CONFIG.load(deps.storage)?;
    ensure_eq!(info.sender, cfg.owner, ContractError::NotOwner);

    // ensure same denom and amount is less than balance, and deduct it
    if amount.denom != cfg.balance.denom {
        return Err(ContractError::InvalidDenom(cfg.balance.denom));
    }

    cfg.balance.amount = cfg
        .balance
        .amount
        .checked_sub(amount.amount)
        .map_err(|_| ContractError::InsufficientBalance(amount.amount))?;

    CONFIG.save(deps.storage, &cfg)?;

    // // send a message (using stargate to send from a different from_address not this contract)
    // let send = MsgSend {
    //     from_address: cfg.issuer.into(),
    //     to_address: recipient,
    //     amount: vec![amount.into()],
    // };
    // let msg = CosmosMsg::Stargate {
    //     type_url: "/cosmos.bank.v1beta1.MsgSend".to_string(),
    //     value: Binary(send.to_proto_bytes()),
    // };

    let spend_from_issuer = Any {
        type_url: "/cosmwasm.wasm.v1.MsgExecuteContract".to_string(),
        value: MsgExecuteContract {
            sender: env.contract.address.to_string(),
            // issuer address
            contract: cfg.issuer.to_string(),
            msg: to_binary(&abstract_giftcard_issuer::msg::ExecuteMsg::Module(abstract_giftcard_issuer::msg::AppExecuteMsg::Spend {
                recipient: recipient.clone(),
                amount: amount.clone(),
            })).unwrap().to_vec(),
            funds: vec![],
        }.to_proto_bytes(),
    };

    let spend = MsgExec {
        grantee: env.contract.address.to_string(),
        msgs: vec![
            spend_from_issuer
        ]
    };

    let msg = CosmosMsg::Stargate {
        type_url: "/cosmos.authz.v1beta1.MsgExec".to_string(),
        value: Binary(spend.to_proto_bytes()),
    };

    Ok(Response::new().add_message(msg))
}

#[cfg_attr(feature = "export", entry_point)]
#[cfg_attr(feature = "interface", cw_orch::interface_entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => {
            let cfg = CONFIG.load(deps.storage)?;
            let cfg = ConfigResponse {
                owner: cfg.owner.into(),
                issuer: cfg.issuer.into(),
                balance: cfg.balance,
            };
            to_binary(&cfg)
        }
    }
}
