use crate::msg::AppMigrateMsg;
use crate::replies::REPLY_ID_INIT;
use crate::{
    error::AppError,
    handlers,
    msg::{AppExecuteMsg, AppInstantiateMsg, AppQueryMsg},
    replies,
};
use abstract_app::{AppContract};
use abstract_core::objects::dependency::StaticDependency;
use cosmwasm_std::{Empty, Response};

/// The version of your app
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
/// The id of the app
pub const APP_ID: &str = "abstract:giftcard-issuer";

/// The type of the result returned by your app's entry points.
pub type AppResult<T = Response> = Result<T, AppError>;

/// The type of the app that is used to build your app and access the Abstract SDK features.
pub type GiftcardIssuerApp =
    AppContract<AppError, AppInstantiateMsg, AppExecuteMsg, AppQueryMsg, AppMigrateMsg, Empty>;

const DEX_DEPENDENCY: StaticDependency = StaticDependency::new(
    abstract_dex_adapter::EXCHANGE,
    &[abstract_dex_adapter::contract::CONTRACT_VERSION],
);

const APP: GiftcardIssuerApp = GiftcardIssuerApp::new(APP_ID, APP_VERSION, None)
    .with_instantiate(handlers::instantiate_handler)
    .with_execute(handlers::execute_handler)
    .with_query(handlers::query_handler)
    // .with_migrate(handlers::migrate_handler)
    // .with_receive(handlers::receive_handler)
    .with_replies(&[(REPLY_ID_INIT, replies::reply_init)])
    .with_dependencies(&[DEX_DEPENDENCY]);

// Export handlers
#[cfg(feature = "export")]
abstract_app::export_endpoints!(APP, GiftcardIssuerApp);

#[cfg(feature = "interface")]
abstract_app::cw_orch_interface!(APP, GiftcardIssuerApp, GiftcardIssuer);
