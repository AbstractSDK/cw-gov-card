use cosmwasm_std::{Binary, CosmosMsg, DepsMut, Env, Reply, Response};
use cw_utils::parse_reply_instantiate_data;
use osmosis_std::shim::Any;
use osmosis_std::types::cosmos::authz::v1beta1::{Grant, MsgGrant};
use osmosis_std::types::cosmos::bank::v1beta1::SendAuthorization;
use crate::contract::{AppResult, GiftcardIssuerApp};
use crate::state::REPLY_INFO;

pub fn reply_init(deps: DepsMut, env: Env, app: GiftcardIssuerApp, reply: Reply) -> AppResult {
 // TODO!

    let created = parse_reply_instantiate_data(reply)?;
    let gift_card = created.contract_address;

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
