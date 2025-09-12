use multiversx_sc_scenario::rust_biguint;
use order_book::{actors::executor::RouterEndpointName, storage::order::OrderDuration};

use crate::order_book_setup::{
    ExecuteOrdersArg, OrderBookSetup, UnmanagedSwapOperationType, TOKEN_IDS, USER_BALANCE,
};

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

#[test]
fn cancel_order_test() {
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

    setup.call_cancel_order(0).assert_ok();
    setup.b_mock.borrow_mut().check_esdt_balance(
        &setup.user,
        TOKEN_IDS[0],
        &rust_biguint!(USER_BALANCE),
    );
}

#[test]
fn execute_order_part_test() {
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

    setup.call_execute_orders(&vec![ExecuteOrdersArg {
        order_id,
        amount_to_swap: 250,
        swap_args: vec![UnmanagedSwapOperationType {
            pair_address: setup.pair_setups[0].pair_wrapper.address_ref().clone(),
            endpoint_name: RouterEndpointName::FixedInput,
            output_token_id: TOKEN_IDS[1].to_vec(),
        }],
    }]);

    // TODO: Check balances
}
