use std::str::FromStr;
use abstract_core::{app::BaseInstantiateMsg, objects::gov_type::GovernanceDetails};
use abstract_core::objects::AssetEntry;
use abstract_dex_adapter::{contract::CONTRACT_VERSION, msg::DexInstantiateMsg, EXCHANGE};
use abstract_interface::{Abstract, AbstractAccount, AppDeployer, VCExecFns, *};
use abstract_giftcard_issuer::{
    contract::{APP_ID, APP_VERSION},
    msg::{AppInstantiateMsg, ConfigResponse, InstantiateMsg},
    *,
    GiftcardIssuer,
};
use abstract_core::ans_host::ExecuteMsgFns as AnsExecMsgFns;
use abstract_dex_adapter::interface::DexAdapter;
use abstract_testing::prelude::{TEST_ACCOUNT_ID, TEST_NAMESPACE};
// Use prelude to get all the necessary imports
use cw_orch::{anyhow, deploy::Deploy, interface, prelude::*};
use speculoos::prelude::*;

use abstract_dex_adapter::msg::ExecuteMsg as FullDexExecuteMsg;
use abstract_dex_adapter::msg::InstantiateMsg as FullDexInstantiateMsg;
use abstract_dex_adapter::msg::QueryMsg as FullDexQueryMsg;

use cosmwasm_std::{Addr, coins, Decimal};
use cw_asset::AssetInfoUnchecked;
use cw_orch::osmosis_test_tube::osmosis_test_tube::Account;
use cw_giftcard::CwGiftcard;

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
fn setup() -> anyhow::Result<(AbstractAccount<OsmosisTestTube>, Abstract<OsmosisTestTube>, GiftcardIssuer<OsmosisTestTube>, CwGiftcard<OsmosisTestTube>)> {
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
fn setup_giftcard(OsmosisTestTube: &OsmosisTestTube) -> CwGiftcard<OsmosisTestTube> {
    let giftcard = cw_giftcard::CwGiftcard::new("giftcard", OsmosisTestTube.clone());
    giftcard.upload().unwrap();

    giftcard
}

// Uploads and returns the giftcard issuer
fn setup_giftcard_issuer(OsmosisTestTube: &OsmosisTestTube) -> GiftcardIssuer<OsmosisTestTube> {
    let giftcard_issuer = GiftcardIssuer::new(APP_ID, OsmosisTestTube.clone());

    // deploy the giftcard issuer module
    giftcard_issuer.deploy(APP_VERSION.parse().unwrap()).unwrap();

    giftcard_issuer
}


// Returns an Abstract with the necessary setup
fn setup_abstract(OsmosisTestTube: &OsmosisTestTube) -> Abstract<OsmosisTestTube> {
    let abstr_deployment = Abstract::deploy_on(OsmosisTestTube.clone(), Empty {}).unwrap();

    abstr_deployment
}

const HACK_DEX_ADAPTER_VERSION: &'static str = "0.17.1";

// Returns a dex adapter with the necessary setup
fn setup_dex_adapter(OsmosisTestTube: &OsmosisTestTube) -> HackDexAdapter<OsmosisTestTube> {
    let mut dex_adapter = HackDexAdapter::new(
        EXCHANGE,
        OsmosisTestTube.clone(),
    );
    dex_adapter.deploy(
        HACK_DEX_ADAPTER_VERSION.parse().unwrap(),
        DexInstantiateMsg {
            recipient_account: 0,
            swap_fee: Decimal::percent(1),
        },
    ).unwrap();

    dex_adapter
}

// let signing_account = abstr_deployment.account_factory.get_chain().clone().init_account(coins(1000, ISSUE_DENOM))?;

// Returns an account with the necessary setup
fn setup_new_account(abstr_deployment: &Abstract<OsmosisTestTube>, namespace: impl ToString) -> anyhow::Result<AbstractAccount<OsmosisTestTube>> {
    // TODO: might need to move this
    let signing_account = abstr_deployment
        .account_factory
        .get_chain()
        .sender();

    // Create a new account to install the app onto
    let account =
        abstr_deployment
            .account_factory
            .create_default_account(GovernanceDetails::Monarchy {
                monarch: signing_account.into_string(),
            }).unwrap();

    // claim the namespace so app can be deployed
    abstr_deployment
        .version_control
        .claim_namespace(account.id().unwrap(), namespace.to_string()).unwrap();

    Ok(account)
}

fn setup_default_assets(abstr: &Abstract<OsmosisTestTube>) {
    // register juno as an asset
    abstr
        .ans_host
        .update_asset_addresses(vec![(ISSUE_ASSET.to_string(), AssetInfoUnchecked::from_str(&format!("native:{}", ISSUE_DENOM)).unwrap())], vec![]).unwrap();
}

fn install_modules_on_account(abstr: &Abstract<OsmosisTestTube>, account: &AbstractAccount<OsmosisTestTube>, issuer: &GiftcardIssuer<OsmosisTestTube>, giftcard: CwGiftcard<OsmosisTestTube>) -> anyhow::Result<()> {

    install_dex_on_account(account)?;
    install_giftcard_issuer_on_account(abstr, account, issuer, AppInstantiateMsg {
        issue_asset: AssetEntry::from(ISSUE_ASSET),
        giftcard_module_id: giftcard.code_id()?,
        // giftcard_module_id: "abstract:giftcard".to_string(),
    })?;

    Ok(())
}

fn install_dex_on_account(account: &AbstractAccount<OsmosisTestTube>) -> anyhow::Result<()> {
    // install exchange module as it's a dependency
    account.install_module(EXCHANGE, &Empty {}, None)?;

    Ok(())
}

fn install_giftcard_issuer_on_account(abstr: &Abstract<OsmosisTestTube>, account: &AbstractAccount<OsmosisTestTube>, issuer: &GiftcardIssuer<OsmosisTestTube>, init_msg: AppInstantiateMsg) -> anyhow::Result<()> {
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
            issue_asset: AssetEntry::from(ISSUE_ASSET),
            issue_denom: ISSUE_DENOM.to_string(),
            giftcard_id: giftcard.code_id()?,
        }
    );
    Ok(())
}

#[test]
fn asset_not_found() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (_account, _abstr, gc_issuer, _giftcard) = setup()?;

    let account = setup_new_account(&_abstr, "two")?;
    install_dex_on_account(&account)?;
    let install_res = install_giftcard_issuer_on_account(&_abstr, &account, &gc_issuer,
                                                         AppInstantiateMsg {
            issue_asset: AssetEntry::from("not_found"),
            giftcard_module_id: gc_issuer.code_id()?,
        });

    assert_that!(install_res).is_err();
    Ok(())
}

#[test]
fn pay_for_gc() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (account, _abstr, gc_issuer, giftcard) = setup()?;

    gc_issuer.issue(None, &coins(5u128, ISSUE_DENOM))?;
    Ok(())
}
