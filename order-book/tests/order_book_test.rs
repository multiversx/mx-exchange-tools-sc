use order_book::storage::order::OrderDuration;

use crate::order_book_setup::{OrderBookSetup, TOKEN_IDS};

pub mod order_book_setup;
pub mod pair_setup;
pub mod router_setup;

#[test]
fn setup_test() {
    let _ = OrderBookSetup::new(
        pair::contract_obj,
        router::contract_obj,
        order_book::contract_obj,
    );
}

#[test]
fn create_order_test() {
    let setup = OrderBookSetup::new(
        pair::contract_obj,
        router::contract_obj,
        order_book::contract_obj,
    );

    let (tx_result, order_id) = setup.call_create_order(
        TOKEN_IDS[0],
        1_000,
        TOKEN_IDS[1],
        1_500,
        OrderDuration::Minutes(10),
        Some(1_000), // 10%
    );
    tx_result.assert_ok();
    assert_eq!(order_id, 0);
}
