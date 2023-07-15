use abstract_giftcard_issuer::contract::APP_ID;
use abstract_giftcard_issuer::GiftcardIssuer;
use cw_orch::prelude::*;

// consts for testing
const ADMIN: &str = "admin";

#[test]
fn successful_wasm() {
    // Create a sender
    let sender = Addr::unchecked(ADMIN);
    // Create the mock
    let mock = Mock::new(&sender);

    // Construct the counter interface
    let contract = GiftcardIssuer::new(APP_ID, mock);

    contract.wasm();
}
