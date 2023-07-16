use std::ops::Add;
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use abstract_core::{ABSTRACT_EVENT_TYPE, ans_host::ExecuteMsgFns as AnsExecMsgFns, app::BaseInstantiateMsg, objects::gov_type::GovernanceDetails};
use abstract_dex_adapter::{
    EXCHANGE,
    msg::{
        DexInstantiateMsg, ExecuteMsg as FullDexExecuteMsg,
        InstantiateMsg as FullDexInstantiateMsg, QueryMsg as FullDexQueryMsg,
    },
};
use abstract_interface::{*, Abstract, AbstractAccount, AppDeployer, VCExecFns};
use abstract_testing::prelude::TEST_NAMESPACE;
use cosmos_sdk_proto::cosmos::base;
use cosmos_sdk_proto::cosmos::distribution::v1beta1::CommunityPoolSpendProposal;
use cosmwasm_std::{coin, coins, Decimal, VoteOption};
use cw_asset::AssetInfoUnchecked;
// Use prelude to get all the necessary imports
use cw_orch::{anyhow, deploy::Deploy, interface, prelude::*};
use cw_orch::osmosis_test_tube::osmosis_test_tube::Account;
use osmosis_std::shim::{Any, Timestamp};
use osmosis_std::types::cosmos::authz::v1beta1::{GenericAuthorization, Grant, MsgGrant, MsgGrantResponse};
use osmosis_test_tube::{Gov, Module, Runner};
use osmosis_test_tube::cosmrs::proto::cosmos::gov::v1beta1::{QueryParamsRequest, QueryProposalRequest};
use osmosis_test_tube::cosmrs::proto::cosmos::params::v1beta1::{ParamChange, ParameterChangeProposal};
use speculoos::prelude::*;

use abstract_giftcard_issuer::{
    *,
    contract::{APP_ID, APP_VERSION},
    GiftcardIssuer, msg::{AppInstantiateMsg, ConfigResponse, InstantiateMsg},
};
use cw_gov_card::{CwGiftcardExecuteFns, CwGovCard};

// consts for testing
const ADMIN: &str = "admin";

const ISSUE_ASSET: &str = "osmo>osmo";
const ISSUE_DENOM: &'static str = "uosmo";

#[interface(FullDexInstantiateMsg, FullDexExecuteMsg, FullDexQueryMsg, Empty)]
pub struct HackDexAdapter<Chain>;

// Implement deployer trait
impl<Chain: CwEnv> AdapterDeployer<Chain, DexInstantiateMsg> for HackDexAdapter<Chain> {}

impl<Chain: CwEnv> Uploadable for HackDexAdapter<Chain> {
    fn wrapper(&self) -> <Mock as TxHandler>::ContractSource {
        Box::new(ContractWrapper::new_with_empty(
            abstract_dex_adapter::contract::execute,
            abstract_dex_adapter::contract::instantiate,
            abstract_dex_adapter::contract::query,
        ))
    }
    fn wasm(&self) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("abstract_dex_adapter")
            .unwrap()
    }
}

/// Set up the test environment with the contract installed
fn setup() -> anyhow::Result<(
    AbstractAccount<OsmosisTestTube>,
    Abstract<OsmosisTestTube>,
    GiftcardIssuer<OsmosisTestTube>,
    CwGovCard<OsmosisTestTube>,
)> {
    // Download the adapter wasm
    // Create the OsmosisTestTube
    let test_tube = OsmosisTestTube::new(coins(1_000_000_000_000, ISSUE_DENOM));

    let abstr = setup_abstract(&test_tube);
    let dex_adapter = setup_dex_adapter(&test_tube);
    let giftcard_issuer = setup_giftcard_issuer(&test_tube);
    let giftcard = setup_giftcard(&test_tube);

    let account = setup_new_account(&abstr, TEST_NAMESPACE)?;
    setup_default_assets(&abstr);
    install_modules_on_account(&abstr, &account, &giftcard_issuer, giftcard.clone())?;

    Ok((account, abstr, giftcard_issuer, giftcard))
}

// Uploads and returns the giftcard contract
fn setup_giftcard(tube: &OsmosisTestTube) -> CwGovCard<OsmosisTestTube> {
    let giftcard = cw_gov_card::CwGovCard::new("giftcard", tube.clone());
    giftcard.upload().unwrap();

    giftcard
}

// Uploads and returns the giftcard issuer
fn setup_giftcard_issuer(tube: &OsmosisTestTube) -> GiftcardIssuer<OsmosisTestTube> {
    let giftcard_issuer = GiftcardIssuer::new(APP_ID, tube.clone());

    // deploy the giftcard issuer module
    giftcard_issuer
        .deploy(APP_VERSION.parse().unwrap())
        .unwrap();

    giftcard_issuer
}

// Returns an Abstract with the necessary setup
fn setup_abstract(tube: &OsmosisTestTube) -> Abstract<OsmosisTestTube> {
    let abstr_deployment = Abstract::deploy_on(tube.clone(), Empty {}).unwrap();

    abstr_deployment
}

const HACK_DEX_ADAPTER_VERSION: &'static str = "0.17.1";

// Returns a dex adapter with the necessary setup
fn setup_dex_adapter(tube: &OsmosisTestTube) -> HackDexAdapter<OsmosisTestTube> {
    let mut dex_adapter = HackDexAdapter::new(EXCHANGE, tube.clone());
    dex_adapter
        .deploy(
            HACK_DEX_ADAPTER_VERSION.parse().unwrap(),
            DexInstantiateMsg {
                recipient_account: 0,
                swap_fee: Decimal::percent(1),
            },
        )
        .unwrap();

    dex_adapter
}

// let signing_account = abstr_deployment.account_factory.get_chain().clone().init_account(coins(1000, ISSUE_DENOM))?;

// Returns an account with the necessary setup
fn setup_new_account(
    abstr_deployment: &Abstract<OsmosisTestTube>,
    namespace: impl ToString,
) -> anyhow::Result<AbstractAccount<OsmosisTestTube>> {
    // TODO: might need to move this
    let signing_account = abstr_deployment.account_factory.get_chain().sender();

    // Create a new account to install the app onto
    let account = abstr_deployment
        .account_factory
        .create_default_account(GovernanceDetails::Monarchy {
            monarch: signing_account.into_string(),
        })
        .unwrap();

    // claim the namespace so app can be deployed
    abstr_deployment
        .version_control
        .claim_namespace(account.id().unwrap(), namespace.to_string())
        .unwrap();

    // register base asset!
    // account.proxy.call_as(&abstr_deployment.account_factory.get_chain().sender).update_assets(vec![(AssetEntry::from(ISSUE_ASSET), UncheckedPriceSource::None)], vec![]).unwrap();

    Ok(account)
}

fn setup_default_assets(abstr: &Abstract<OsmosisTestTube>) {
    // register juno as an asset
    abstr
        .ans_host
        .update_asset_addresses(
            vec![(
                ISSUE_ASSET.to_string(),
                AssetInfoUnchecked::from_str(&format!("native:{}", ISSUE_DENOM)).unwrap(),
            )],
            vec![],
        )
        .unwrap();
}

fn install_modules_on_account(
    abstr: &Abstract<OsmosisTestTube>,
    account: &AbstractAccount<OsmosisTestTube>,
    issuer: &GiftcardIssuer<OsmosisTestTube>,
    giftcard: CwGovCard<OsmosisTestTube>,
) -> anyhow::Result<()> {
    install_dex_on_account(account)?;
    install_giftcard_issuer_on_account(
        abstr,
        account,
        issuer,
        AppInstantiateMsg {
            giftcard_module_id: giftcard.code_id()?,
            // giftcard_module_id: "abstract:giftcard".to_string(),
        },
    )?;

    Ok(())
}

fn install_dex_on_account(account: &AbstractAccount<OsmosisTestTube>) -> anyhow::Result<()> {
    // install exchange module as it's a dependency
    account.install_module(EXCHANGE, &Empty {}, None)?;

    Ok(())
}

fn install_giftcard_issuer_on_account(
    abstr: &Abstract<OsmosisTestTube>,
    account: &AbstractAccount<OsmosisTestTube>,
    issuer: &GiftcardIssuer<OsmosisTestTube>,
    init_msg: AppInstantiateMsg,
) -> anyhow::Result<()> {
    account.install_module(
        APP_ID,
        &InstantiateMsg {
            base: BaseInstantiateMsg {
                ans_host_address: abstr.ans_host.addr_str()?,
            },
            module: init_msg,
        },
        None,
    )?;

    let modules = account.manager.module_infos(None, None)?;
    issuer.set_address(&modules.module_infos[1].address);

    Ok(())
}

#[test]
fn successful_install() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (_account, _abstr, gc_issuer, giftcard) = setup()?;

    let config = gc_issuer.config()?;
    assert_eq!(
        config,
        ConfigResponse {
            giftcard_id: giftcard.code_id()?,
        }
    );
    Ok(())
}

/*#[test]
fn asset_not_found() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (_account, _abstr, gc_issuer, _giftcard) = setup()?;

    let account = setup_new_account(&_abstr, "two")?;
    install_dex_on_account(&account)?;
    let install_res = install_giftcard_issuer_on_account(
        &_abstr,
        &account,
        &gc_issuer,
        AppInstantiateMsg {
            giftcard_module_id: gc_issuer.code_id()?,
        },
    );

    assert_that!(install_res).is_err();
    Ok(())
}*/

#[test]
fn post_bribe() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (account, _abstr, gc_issuer, giftcard) = setup()?;
    let tube = gc_issuer.get_chain();
    let sender = tube.sender();
    println!("sender: {:?}", sender);

    // let issue_res = gc_issuer.call_as(&buyer).issue(None, &coins(500u128, ISSUE_DENOM));
    let collateral = coin(500u128, ISSUE_DENOM);
    let price = coin(1000u128, ISSUE_DENOM);

    let issue_res = gc_issuer.issue(collateral.clone(), price.clone(), None, &[collateral.clone()]);

    let issue_res = assert_that!(issue_res).is_ok().subject.to_owned();
    println!("{:?}", issue_res);

    let voter_card_addr = issue_res.event_attr_value(ABSTRACT_EVENT_TYPE, "voter_card")?;

    let mut voter_card = CwGovCard::new("cw-gov-card", tube.clone());
    voter_card.set_address(&Addr::unchecked(voter_card_addr.clone()));
    println!("voter_card_addr: {:?}", voter_card_addr);
    let voter_card_config = cw_gov_card::types::QueryMsgFns::config(&voter_card)?;
    // check that the voter card is owned by the issuer
    assert_that!(voter_card_config.owner).is_equal_to(gc_issuer.address()?.to_string());

    // check that the voter card has the collateral
    assert_that!(gc_issuer.get_chain().clone().query_balance(&voter_card_addr, ISSUE_DENOM)).is_ok().is_equal_to(collateral.amount);

    // buy our own bribe (should be another buyer, but bug prevents)
    let bribe_res = gc_issuer.bribe(sender.to_string(), &[price]);
    assert_that!(bribe_res).is_ok();

    let voter_card_config = cw_gov_card::types::QueryMsgFns::config(&voter_card)?;
    // check that the voter card is owned by the briber
    assert_that!(voter_card_config.owner).is_equal_to(sender.to_string());

    let proposer = tube.init_accounts(coins(100000000000, ISSUE_DENOM), 1)?.pop().unwrap();


    let proposal = {
        let (prop, voting_period) = {
            let mut app_binding = tube.app.borrow_mut();
            let gov = Gov::new(&mut *app_binding);

            let mut sender_binding = tube.sender.borrow_mut();
            let proposal_submission = gov.submit_executable_proposal("/cosmos.distribution.v1beta1.CommunityPoolSpendProposal".to_string(), CommunityPoolSpendProposal {
                title: "aoeu".to_string(),
                description: "aoeu".to_string(),
                recipient: sender.to_string(),
                amount: vec![base::v1beta1::Coin {
                    denom: "uosmo".to_string(),
                    amount: 1000000.to_string(),
                }],
            },
             vec![],
                                                                     proposer.address(),
                                                                     &proposer,
            );



            // query params
            let params = gov.query_params(&QueryParamsRequest {
                params_type: "voting".to_string(),
            })?;

            // get voting period
            let voting_period = params
                .voting_params
                .expect("voting params must exist")
                .voting_period
                .expect("voting period must exist");

            (proposal_submission, voting_period)
        };

        println!("prop_submission: {:?}", prop);


        // increase time to pass voting period
        tube.wait_seconds(1)?;

        let mut app_binding = tube.app.borrow_mut();
        let gov = Gov::new(&mut *app_binding);

        let query = gov.query_proposal(&QueryProposalRequest{
            proposal_id: 1,
        })?;

        prop
    };

    let proposal_id = assert_that!(proposal).is_ok().subject.data.proposal_id;

    // vote on the proposal as the nft
    // should fail the first time without authz
    let vote_res = voter_card.cast_vote(proposal_id, VoteOption::Yes);
    assert_that!(vote_res).is_err().matches(|e| {
        e.to_string().contains("unauthorized")
    });


    let tube = tube.clone();
    // authorize the voter card to vote on behalf of the issuer
    let authz_res = {
        let mut app_binding = tube.app.borrow_mut();
        let mut sender_binding = tube.sender.borrow_mut();

        let now = SystemTime::now();
        let since_the_epoch = now
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        let in_seconds = since_the_epoch.add(Duration::new((60 * 6* 24) as u64, 0)).as_secs() as i64;

        (&mut *app_binding).execute::<osmosis_std::types::cosmos::authz::v1beta1::MsgGrant, MsgGrantResponse>(MsgGrant {
            granter: sender.to_string(),
            grantee: voter_card_addr.clone(),
            grant: Some(Grant {
                authorization: Some(Any {
                    type_url: "/cosmos.authz.v1beta1.GenericAuthorization".to_string(),
                    value: GenericAuthorization {
                        msg: "/cosmos.gov.v1beta1.MsgVote".to_string(),
                    }.to_proto_bytes(),
                }),
                expiration: Some(Timestamp {
                    seconds: in_seconds,
                    nanos: 0,
                }),
            }),
        },       "/cosmos.authz.v1beta1.MsgGrant", &mut *sender_binding)
    };

    assert_that!(authz_res).is_ok();

    // TODO: 1: inactive proposal... why does it say inactive?
    let vote_res = voter_card.cast_vote(proposal_id, VoteOption::Yes);
    assert_that!(vote_res).is_ok();

    Ok(())
}
