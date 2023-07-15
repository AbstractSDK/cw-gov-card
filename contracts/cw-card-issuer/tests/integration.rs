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
// Use prelude to get all the necessary imports
use cw_orch::{anyhow, deploy::Deploy, prelude::*};

use cosmwasm_std::{Addr, Decimal};
use cw_asset::AssetInfoUnchecked;
use cw_giftcard::CwGiftcard;

// consts for testing
const ADMIN: &str = "admin";

const ISSUE_ASSET: &str = "juno>juno";

/// Set up the test environment with the contract installed
fn setup() -> anyhow::Result<(AbstractAccount<Mock>, Abstract<Mock>, GiftcardIssuer<Mock>, CwGiftcard<Mock>)> {
    // Create a sender
    let sender = Addr::unchecked(ADMIN);
    // Create the mock
    let mock = Mock::new(&sender);

    let giftcard = cw_giftcard::CwGiftcard::new("giftcard", mock.clone());
    giftcard.upload()?;

    let giftcard_issuer = GiftcardIssuer::new(APP_ID, mock.clone());

    // Deploy Abstract to the mock
    let abstr_deployment = Abstract::deploy_on(mock.clone(), Empty {})?;

    // Deploy the DEX adapter
    let dex_adapter = abstract_dex_adapter::interface::DexAdapter::new(
        abstract_dex_adapter::EXCHANGE,
        mock.clone(),
    );
    dex_adapter.deploy(
        CONTRACT_VERSION.parse().unwrap(),
        DexInstantiateMsg {
            recipient_account: 0,
            swap_fee: Decimal::percent(1),
        },
    )?;

    // Create a new account to install the app onto
    let account =
        abstr_deployment
            .account_factory
            .create_default_account(GovernanceDetails::Monarchy {
                monarch: ADMIN.to_string(),
            })?;

    // claim the namespace so app can be deployed
    abstr_deployment
        .version_control
        .claim_namespace(1, "my-namespace".to_string())?;

    // deploy the giftcard issuer module
    giftcard_issuer.deploy(APP_VERSION.parse()?)?;

    // register juno as an asset
    abstr_deployment
        .ans_host
        .update_asset_addresses(vec![(ISSUE_ASSET.to_string(), AssetInfoUnchecked::from_str(&format!("native:{}", ISSUE_DENOM)).unwrap())], vec![])?;

    // install exchange module as it's a dependency
    account.install_module(EXCHANGE, &Empty {}, None)?;

    account.install_module(
        APP_ID,
        &InstantiateMsg {
            base: BaseInstantiateMsg {
                ans_host_address: abstr_deployment.ans_host.addr_str()?,
            },
            module: AppInstantiateMsg {
                issue_asset: AssetEntry::from(ISSUE_ASSET),
                giftcard_module_id: giftcard.code_id()?,
                // giftcard_module_id: "abstract:giftcard".to_string(),
            },
        },
        None,
    )?;

    let modules = account.manager.module_infos(None, None)?;
    giftcard_issuer.set_address(&modules.module_infos[1].address);

    Ok((account, abstr_deployment, giftcard_issuer, giftcard))
}

const ISSUE_DENOM: &'static str = "ujuno";

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
