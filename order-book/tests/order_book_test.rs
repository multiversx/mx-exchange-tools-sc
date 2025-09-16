use multiversx_sc_scenario::{managed_address, managed_biguint, managed_token_id, rust_biguint};
use order_book::{
    actors::executor::RouterEndpointName,
    storage::order::{Order, OrderDuration, OrderModule},
};

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

    let user_addr = setup.user.clone();
    setup
        .b_mock
        .borrow_mut()
        .execute_query(&setup.order_book_wrapper, |sc| {
            let actual_order = sc.orders(0).get();
            let expected_order = Order {
                maker: managed_address!(&user_addr),
                input_token: managed_token_id!(TOKEN_IDS[0]),
                output_token: managed_token_id!(TOKEN_IDS[1]),
                initial_input_amount: managed_biguint!(1_000),
                current_input_amount: managed_biguint!(1_000),
                min_total_output: managed_biguint!(1_500),
                executor_fee: 1_000,
                creation_timestamp: 0,
                expiration_timestamp: 10 * 60,
            };

            assert_eq!(actual_order, expected_order);
        })
        .assert_ok();
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

    // First pair is A:B with 1:2 ratio
    // 250 input to 500 output -> Total = 124 + 375 = 499 ~= 500 (minus pair fees)
    setup
        .b_mock
        .borrow()
        .check_esdt_balance(&setup.owner, TOKEN_IDS[1], &rust_biguint!(124));
    setup.b_mock.borrow().check_esdt_balance(
        &setup.user,
        TOKEN_IDS[1],
        &rust_biguint!(USER_BALANCE + 375),
    );
    setup.b_mock.borrow().check_esdt_balance(
        setup.order_book_wrapper.address_ref(),
        TOKEN_IDS[1],
        &rust_biguint!(0),
    );

    setup.b_mock.borrow().check_esdt_balance(
        &setup.user,
        TOKEN_IDS[0],
        &rust_biguint!(USER_BALANCE - 1_000),
    );

    // check internal order structure
    let user_addr = setup.user.clone();
    setup
        .b_mock
        .borrow_mut()
        .execute_query(&setup.order_book_wrapper, |sc| {
            let actual_order = sc.orders(0).get();
            let expected_order = Order {
                maker: managed_address!(&user_addr),
                input_token: managed_token_id!(TOKEN_IDS[0]),
                output_token: managed_token_id!(TOKEN_IDS[1]),
                initial_input_amount: managed_biguint!(1_000),
                current_input_amount: managed_biguint!(1_000 - 250),
                min_total_output: managed_biguint!(1_500),
                executor_fee: 1_000,
                creation_timestamp: 0,
                expiration_timestamp: 10 * 60,
            };

            assert_eq!(actual_order, expected_order);
        })
        .assert_ok();

    // check SC balance is the same as stored in order struct
    setup.b_mock.borrow().check_esdt_balance(
        setup.order_book_wrapper.address_ref(),
        TOKEN_IDS[0],
        &rust_biguint!(1_000 - 250),
    );
}
