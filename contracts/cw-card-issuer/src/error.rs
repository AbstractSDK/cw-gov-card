use abstract_app::AppError as AbstractAppError;
use abstract_core::AbstractError;
use abstract_sdk::AbstractSdkError;
use cosmwasm_std::{Coin, StdError};
use cw_asset::Asset;
use cw_utils::{ParseReplyError, PaymentError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum AppError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Abstract(#[from] AbstractError),

    #[error("{0}")]
    AbstractSdk(#[from] AbstractSdkError),

    // #[error("{0}")]
    // Asset(#[from] AssetError),

    #[error("{0}")]
    Admin(#[from] cw_controllers::AdminError),

    #[error("{0}")]
    DappError(#[from] AbstractAppError),

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("{0}")]
    Parse(#[from] ParseReplyError),

    #[error("Only native tokens are supported.")]
    OnlyNativeSupported,

    #[error("Gitfcard module not found.")]
    ModuleNotFound,

    #[error("Only spend via authz is supported. Sender: {0}")]
    OnlySpendViaAuthz(String),

    #[error("Insufficient funds. Balance: {balance:?}, required: {required:?}")]
    InsufficientFunds {
        balance: Asset,
        required: Coin,
    },
}
