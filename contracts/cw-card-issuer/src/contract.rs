#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdResult, SubMsg, WasmMsg,
};
use cw2::set_contract_version;
use cw_utils::{must_pay, parse_reply_instantiate_data};
use osmosis_std::shim::Any;
use osmosis_std::types::cosmos::authz::v1beta1::{Grant, MsgGrant};
use osmosis_std::types::cosmos::bank::v1beta1::SendAuthorization;

use crate::types::{
    Config, ConfigResponse, ContractError, ExecuteMsg, GiftCardInstantiateMsg, InstantiateMsg,
    QueryMsg, CONFIG, REPLY_INFO,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-card-issuer";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const REPLY_ID_INIT: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // TODO: validate denom not zero
    let cfg = Config {
        denom: msg.denom,
        giftcard_id: msg.giftcard_id,
    };
    CONFIG.save(deps.storage, &cfg)?;
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Issue { label } => issue(deps, info, label),
    }
}

pub fn issue(
    deps: DepsMut,
    info: MessageInfo,
    label: Option<String>,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    let amount = must_pay(&info, &cfg.denom)?;

    // instantitate giftcard
    let allowance = Coin {
        amount,
        denom: cfg.denom,
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => {
            let cfg = CONFIG.load(deps.storage)?;
            let cfg = ConfigResponse {
                denom: cfg.denom,
                giftcard_id: cfg.giftcard_id,
            };
            to_binary(&cfg)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
    match reply.id {
        // only on success and we just query current state, ignore response data
        REPLY_ID_INIT => {
            let created = parse_reply_instantiate_data(reply)?;
            reply_init(deps, env, created.contract_address)
        }
        _ => Err(ContractError::InvalidReplyId(reply.id)),
    }
}

pub fn reply_init(deps: DepsMut, env: Env, gift_card: String) -> Result<Response, ContractError> {
    // figure out who to send back to
    let allowance = REPLY_INFO.load(deps.storage)?;
    REPLY_INFO.remove(deps.storage);

    // TODO: Issue authz allowance to spend the allowance to the gift_card address
    let send_auth = Any {
        type_url: "/cosmos.bank.v1beta1.SendAuthorization".to_string(),
        value: SendAuthorization {
            spend_limit: vec![allowance.into()],
        }
        .to_proto_bytes(),
    };
    let grant = MsgGrant {
        granter: env.contract.address.to_string(),
        grantee: gift_card,
        grant: Some(Grant {
            authorization: Some(send_auth),
            expiration: None,
        }),
    };
    let msg = CosmosMsg::Stargate {
        type_url: "/cosmos.authz.v1beta1.MsgGrant".to_string(),
        value: Binary(grant.to_proto_bytes()),
    };

    Ok(Response::new().add_message(msg))
}
