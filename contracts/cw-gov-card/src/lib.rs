pub mod contract;
// This is msg, state, error in one
pub mod types;

#[cfg(feature = "interface")]
pub use crate::contract::CwGovCard;
// in lib.rs
#[cfg(feature = "interface")]
pub use crate::types::{ExecuteMsgFns as CwGiftcardExecuteFns, QueryMsgFns as CwGiftcardQueryFns};
