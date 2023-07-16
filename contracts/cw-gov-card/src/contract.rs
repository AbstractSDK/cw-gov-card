#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{ensure_eq, to_binary, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult, ensure, QuerierWrapper, StdError};
use cw2::set_contract_version;
use cw_utils::{nonpayable, one_coin};
use osmosis_std::shim::Any;
use osmosis_std::types::cosmos::authz::v1beta1::MsgExec;
use osmosis_std::types::cosmos::bank::v1beta1::MsgSend;
use osmosis_std::types::cosmos::gov::v1beta1::{GovQuerier, MsgVote, Proposal, ProposalStatus, QueryProposalResponse, Vote, VoteOption};
use cosmwasm_std::VoteOption as CwVoteOption;
use cw_asset::AssetList;
use osmosis_std::types::cosmwasm::wasm::v1::MsgExecuteContract;

use crate::types::{Config, ConfigResponse, ContractError, ExecuteMsg, InstantiateMsg, QueryMsg, CONFIG, VOTES};

// version info for migration info
pub const CONTRACT_NAME: &str = "crates.io:cw-gov-card";
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

    let cfg = Config {
        owner: info.sender,
        collateral: info.funds,
        party: msg.party,
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
        ExecuteMsg::CastVote { proposal_id, vote_option } => {
            let vote_option = match vote_option {
                CwVoteOption::Yes => VoteOption::Yes,
                CwVoteOption::Abstain => VoteOption::Abstain,
                CwVoteOption::No => VoteOption::No,
                CwVoteOption::NoWithVeto => VoteOption::NoWithVeto,
            };
            cast_vote(deps, env, info, proposal_id, vote_option)
        }
        ExecuteMsg::VerifyVoteOutcome { proposal_id } => verify_vote_outcome(deps, env, info, proposal_id),
    }
}

fn verify_vote_outcome(deps: DepsMut, env: Env, info: MessageInfo, proposal_id: u64) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let prop = query_proposal(&deps.querier, proposal_id)?;
    match prop.status {
        3 | 4 | 5 => {
        // ProposalStatus::Passed | ProposalStatus::Rejected | ProposalStatus::Failed => {
            return Err(ContractError::ProposalClosed(proposal_id))
        }
        _ => {}
    }

    let our_vote = VOTES.may_load(deps.storage, proposal_id)?;
    let our_vote = match our_vote {
        Some(v) => v as i32,
        None => return Err(ContractError::DidNotVote(proposal_id))
    };

    let party_vote = query_party_vote(&deps.as_ref(), proposal_id)?;
    // TODO: check all in the future, for now just one
    let party_vote_option = match party_vote.options.get(0) {
        Some(option) => option.option.clone(),
        None => return Err(ContractError::NoPartyVoteOption(proposal_id)), // add this error to your ContractError enum
    };

    let cfg = CONFIG.load(deps.storage)?;
    let collateral_list = AssetList::from(cfg.collateral);

    // If the vote was made in our favor, send the collateral to the party
    let collateral_recipient = if our_vote == party_vote_option {
        // Send the collateral to the party
        cfg.party
    } else {
        // Send the collateral to the owner, because it was made in bad faith
        cfg.owner.to_string()
    };

    let collateral_transfer = collateral_list.transfer_msgs(collateral_recipient)?;
    Ok(Response::new().add_messages(collateral_transfer))
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

pub fn cast_vote(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    proposal_id: u64,
    vote_option: VoteOption,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    let mut cfg = CONFIG.load(deps.storage)?;
    ensure_eq!(info.sender, cfg.owner, ContractError::NotOwner);

    // let prop = query_proposal(&deps.querier, proposal_id)?;
    //
    // ensure_eq!(prop.status, ProposalStatus::VotingPeriod as i32, ContractError::NotVotingPeriod(proposal_id));

    CONFIG.save(deps.storage, &cfg)?;

    VOTES.save(deps.storage, proposal_id, &vote_option)?;

    let vote = Any {
        type_url: "/cosmos.gov.v1beta1.MsgVote".to_string(),
        value: MsgVote {
            proposal_id,
            voter: cfg.party,
            option: vote_option.try_into().unwrap(),
        }.to_proto_bytes()
    };

    let spend = MsgExec {
        grantee: env.contract.address.to_string(),
        msgs: vec![
            vote
        ]
    };

    let msg = CosmosMsg::Stargate {
        type_url: "/cosmos.authz.v1beta1.MsgExec".to_string(),
        value: Binary(spend.to_proto_bytes()),
    };

    Ok(Response::new().add_message(msg))
}

fn query_proposal(querier: &QuerierWrapper, proposal_id: u64) -> Result<Proposal, ContractError> {
    let prop = GovQuerier::new(&querier).proposal(proposal_id)?;

    match prop.proposal {
        Some(proposal) => {
            // ProposalStatus.VotingPeriod
            Ok(proposal)
        },
        None => {
            return Err(ContractError::ProposalNotFound(proposal_id));
        }
    }
}

fn query_party_vote(deps: &Deps, proposal_id: u64,) -> Result<Vote, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;

    let vote_res = GovQuerier::new(&deps.querier).vote(proposal_id, cfg.party)?;

    match vote_res.vote {
        Some(vote) => {
            Ok(vote)
        },
        None => {
            return Err(ContractError::ProposalNotFound(proposal_id));
        }
    }
}

#[cfg_attr(feature = "export", entry_point)]
#[cfg_attr(feature = "interface", cw_orch::interface_entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => {
            let cfg = CONFIG.load(deps.storage)?;
            let cfg = ConfigResponse {
                owner: cfg.owner.into(),
                party: cfg.party.into(),
                collateral: cfg.collateral,
            };
            to_binary(&cfg)
        }
    }
}
