use abstract_sdk::AbstractResponse;
use cosmos_sdk_proto::traits::MessageExt;
use crate::contract::{AppResult, GiftcardIssuerApp};
use crate::state::{GIFTCARDS, REPLY_INFO};
use cosmwasm_std::{Binary, CosmosMsg, DepsMut, Env, Reply, Response};
use cw_utils::parse_reply_instantiate_data;
use osmosis_std::shim::{Any, Duration, Timestamp};
use osmosis_std::types::cosmos::authz::v1beta1::{GenericAuthorization, Grant, MsgGrant};
use osmosis_std::types::cosmos::bank::v1beta1::SendAuthorization;
use osmosis_std::types::cosmwasm::wasm::v1::{
    AcceptedMessageKeysFilter, AllowAllMessagesFilter, ContractExecutionAuthorization,
    ContractGrant, MaxCallsLimit, MaxFundsLimit,
};

pub fn reply_on_issuance(
    deps: DepsMut,
    env: Env,
    app: GiftcardIssuerApp,
    reply: Reply,
) -> AppResult {
    // TODO!

    let created = parse_reply_instantiate_data(reply)?;
    let gift_card = created.contract_address;

    // figure out who to send back to
    let allowance = REPLY_INFO.load(deps.storage)?;
    REPLY_INFO.remove(deps.storage);

    // TODO: Issue authz allowance to spend the allowance to the gift_card address
    // let send_auth = Any {
    //     type_url: "/cosmos.bank.v1beta1.SendAuthorization".to_string(),
    //     value: SendAuthorization {
    //         spend_limit: vec![allowance.into()],
    //     }
    //     .to_proto_bytes(),
    // };

    /*
        export enum FilterTypes {
      All = '/cosmwasm.wasm.v1.AllowAllMessagesFilter',
      Keys = '/cosmwasm.wasm.v1.AcceptedMessageKeysFilter',
      Msgs = '/cosmwasm.wasm.v1.AcceptedMessagesFilter',
    }

    export enum LimitTypes {
      Combined = '/cosmwasm.wasm.v1.CombinedLimit',
      Calls = '/cosmwasm.wasm.v1.MaxCallsLimit',
      Funds = '/cosmwasm.wasm.v1.MaxFundsLimit',
    }

        filter: Some(Any {
                type_url: "/cosmwasm.wasm.v1.AcceptedMessagesFilter".to_string(),
                value: AcceptedMessagesFilter {
                    messages: vec![Any {
                        type_url: "/cosmos.bank.v1beta1.MsgSend".to_string(),
                        value: MsgSend {
                            from_address: env.contract.address.to_string(),
                            to_address: gift_card.to_string(),
                            amount: vec![allowance.into()],
                        }
                        .to_proto_bytes(),
                    }],
                }
                .to_proto_bytes(),
            }),
         */


    let spend_grant = ContractGrant {
        contract: env.contract.address.to_string(),
        filter: Some(Any {
            type_url: "/cosmwasm.wasm.v1.AcceptedMessageKeysFilter".to_string(),
            value: AcceptedMessageKeysFilter {
                keys: vec!["module".to_string()],
            }
            .to_proto_bytes(),
        }),
        limit: Some(Any {
            type_url: "/cosmwasm.wasm.v1.MaxFundsLimit".to_string(),
            value: MaxFundsLimit {
                amounts: vec![allowance.into()],
            }
            .to_proto_bytes(),
        }),
    };


    let spend_grant = ContractGrant {
        contract: env.contract.address.to_string(),
        filter: Some(Any {
            type_url: "/cosmwasm.wasm.v1.AllowAllMessagesFilter".to_string(),
            value: AllowAllMessagesFilter {}.to_proto_bytes(),
        }),
        limit: Some(Any {
            type_url: "/cosmwasm.wasm.v1.MaxCallsLimit".to_string(),
            value: MaxCallsLimit { remaining: 666 }.to_proto_bytes(),
        }),
    };

    let exec_auth = Any {
        type_url: "/cosmwasm.wasm.v1.ContractExecutionAuthorization".to_string(),
        value: ContractExecutionAuthorization {
            grants: vec![spend_grant],
        }
        .to_proto_bytes(),
    };

    let grant = MsgGrant {
        granter: env.contract.address.to_string(),
        grantee: gift_card.clone(),
        grant: Some(Grant {
            authorization: Some(exec_auth),
            expiration: Some(Timestamp {
                seconds: env.block.time.plus_seconds(60 * 60 * 24 * 365).seconds().try_into().unwrap(),
                nanos: 0,
            }),
        }),
    };

    let msg = CosmosMsg::Stargate {
        type_url: "/cosmos.authz.v1beta1.MsgGrant".to_string(),
        value: Binary(grant.to_proto_bytes()),
    };

    GIFTCARDS.insert(deps.storage, &gift_card)?;

    Ok(app.custom_tag_response(Response::new().add_message(msg), "issue_reply", vec![("gift_card", gift_card.as_str())]))
}
