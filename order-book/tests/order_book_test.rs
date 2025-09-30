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

    setup.call_execute_orders(&[ExecuteOrdersArg {
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

    // execute remaining part of order
    setup.call_execute_orders(&[ExecuteOrdersArg {
        order_id,
        amount_to_swap: 750,
        swap_args: vec![UnmanagedSwapOperationType {
            pair_address: setup.pair_setups[0].pair_wrapper.address_ref().clone(),
            endpoint_name: RouterEndpointName::FixedInput,
            output_token_id: TOKEN_IDS[1].to_vec(),
        }],
    }]);

    // First pair is A:B with 1:2 ratio
    // 750 input to 1_500 output -> Total = 374 + 1_125 = 1_499 ~= 1_500 (minus pair fees)
    setup
        .b_mock
        .borrow()
        .check_esdt_balance(&setup.owner, TOKEN_IDS[1], &rust_biguint!(124 + 374));
    setup.b_mock.borrow().check_esdt_balance(
        &setup.user,
        TOKEN_IDS[1],
        &rust_biguint!(USER_BALANCE + 375 + 1_125),
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

    // check internal order structure - should be cleared after full order executed
    setup
        .b_mock
        .borrow_mut()
        .execute_query(&setup.order_book_wrapper, |sc| {
            assert!(sc.orders(0).is_empty());
        })
        .assert_ok();

    // check SC balance is now 0
    setup.b_mock.borrow().check_esdt_balance(
        setup.order_book_wrapper.address_ref(),
        TOKEN_IDS[0],
        &rust_biguint!(0),
    );
}

#[test]
fn execute_order_full_test() {
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

    setup.call_execute_orders(&[ExecuteOrdersArg {
        order_id,
        amount_to_swap: 1_000,
        swap_args: vec![UnmanagedSwapOperationType {
            pair_address: setup.pair_setups[0].pair_wrapper.address_ref().clone(),
            endpoint_name: RouterEndpointName::FixedInput,
            output_token_id: TOKEN_IDS[1].to_vec(),
        }],
    }]);

    // First pair is A:B with 1:2 ratio
    // 1_000 input to 2_000 output -> Total = 499 + 1_500 = 1_999 ~= 2_000 (minus pair fees)
    setup
        .b_mock
        .borrow()
        .check_esdt_balance(&setup.owner, TOKEN_IDS[1], &rust_biguint!(499));
    setup.b_mock.borrow().check_esdt_balance(
        &setup.user,
        TOKEN_IDS[1],
        &rust_biguint!(USER_BALANCE + 1_500),
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

    // check internal order structure - should be cleared after full order executed
    setup
        .b_mock
        .borrow_mut()
        .execute_query(&setup.order_book_wrapper, |sc| {
            assert!(sc.orders(0).is_empty());
        })
        .assert_ok();

    // check SC balance is now 0
    setup.b_mock.borrow().check_esdt_balance(
        setup.order_book_wrapper.address_ref(),
        TOKEN_IDS[0],
        &rust_biguint!(0),
    );
}

#[test]
fn pruner_test() {
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

    // try prune too early
    setup
        .call_prune_expired_order(0)
        .assert_user_error("Order not expired yet");

    setup.b_mock.borrow_mut().set_block_timestamp(5_000_000);

    // prune order ok
    setup.call_prune_expired_order(0).assert_ok();

    setup
        .b_mock
        .borrow()
        .check_esdt_balance(&setup.owner, TOKEN_IDS[0], &rust_biguint!(100));

    setup.b_mock.borrow().check_esdt_balance(
        &setup.user,
        TOKEN_IDS[0],
        &rust_biguint!(USER_BALANCE - 100),
    );
}

#[test]
fn prune_partly_executed_order_test() {
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

    setup.call_execute_orders(&[ExecuteOrdersArg {
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

    setup.b_mock.borrow_mut().set_block_timestamp(5_000_000);

    // prune order ok - total tokens remaining: 750
    setup.call_prune_expired_order(0).assert_ok();

    setup
        .b_mock
        .borrow()
        .check_esdt_balance(&setup.owner, TOKEN_IDS[0], &rust_biguint!(75));

    setup.b_mock.borrow().check_esdt_balance(
        &setup.user,
        TOKEN_IDS[0],
        &rust_biguint!(USER_BALANCE - 250 - 75),
    );
}

#[test]
fn taker_fill_order_buying_input_test() {
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

    // send too few tokens
    setup
        .call_fill_order_p2p_by_buying_input(0, 1_000, TOKEN_IDS[1], 500)
        .assert_user_error("Sent too few tokens");

    // buy part of the tokens
    // Payment 1_406
    // Min maker: 1_125
    // Fees: 1_406 - 1_125 = 281
    let tokens_to_buy = 750;
    let tokens_needed_for_p2p_buy_input =
        setup.call_get_tokens_needed_for_p2p_buy_input(0, tokens_to_buy);
    setup
        .call_fill_order_p2p_by_buying_input(
            0,
            tokens_to_buy,
            TOKEN_IDS[1],
            tokens_needed_for_p2p_buy_input,
        )
        .assert_ok();

    // check balances
    setup
        .b_mock
        .borrow()
        .check_esdt_balance(&setup.treasury, TOKEN_IDS[1], &rust_biguint!(281));

    setup.b_mock.borrow().check_esdt_balance(
        &setup.user,
        TOKEN_IDS[1],
        &rust_biguint!(USER_BALANCE + 1_125),
    );

    setup.b_mock.borrow().check_esdt_balance(
        &setup.taker,
        TOKEN_IDS[0],
        &rust_biguint!(tokens_to_buy),
    );

    // check stored order
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
                current_input_amount: managed_biguint!(1_000 - 750),
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
fn taker_send_extra_tokens_fill_order_buy_input_test() {
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

    // send too few tokens
    setup
        .call_fill_order_p2p_by_buying_input(0, 1_000, TOKEN_IDS[1], 500)
        .assert_user_error("Sent too few tokens");

    // buy part of the tokens
    // Payment 1_406 + 1_000
    // Min maker: 1_125
    // Fees: 1_406 + 1_000 - 1_125 = 1_281
    let tokens_to_buy = 750;
    let tokens_needed_for_p2p_buy_input =
        setup.call_get_tokens_needed_for_p2p_buy_input(0, tokens_to_buy) + 1_000;
    setup
        .call_fill_order_p2p_by_buying_input(
            0,
            tokens_to_buy,
            TOKEN_IDS[1],
            tokens_needed_for_p2p_buy_input,
        )
        .assert_ok();

    // check balances
    setup.b_mock.borrow().check_esdt_balance(
        &setup.treasury,
        TOKEN_IDS[1],
        &rust_biguint!(281 + 1_000),
    );

    setup.b_mock.borrow().check_esdt_balance(
        &setup.user,
        TOKEN_IDS[1],
        &rust_biguint!(USER_BALANCE + 1_125),
    );

    setup.b_mock.borrow().check_esdt_balance(
        &setup.taker,
        TOKEN_IDS[0],
        &rust_biguint!(tokens_to_buy),
    );
}

#[test]
fn taker_fill_order_by_selling_output_test() {
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

    // Payment 1_406
    // Min maker: 1_125
    // Fees: 1_406 - 1_125 = 281
    let tokens_to_buy = 750;
    setup
        .call_fill_order_p2p_by_selling_output(0, TOKEN_IDS[1], 1_406)
        .assert_ok();

    // check balances
    setup
        .b_mock
        .borrow()
        .check_esdt_balance(&setup.treasury, TOKEN_IDS[1], &rust_biguint!(281));

    setup.b_mock.borrow().check_esdt_balance(
        &setup.user,
        TOKEN_IDS[1],
        &rust_biguint!(USER_BALANCE + 1_125),
    );

    setup.b_mock.borrow().check_esdt_balance(
        &setup.taker,
        TOKEN_IDS[0],
        &rust_biguint!(tokens_to_buy),
    );

    // check stored order
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
                current_input_amount: managed_biguint!(1_000 - 750),
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
fn taker_send_extra_tokens_fill_order_by_selling_output_test() {
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

    // Payment: 1_500 + 20% * 1_500 = 1_500 + 300 = 1_800. Extra 1_000 tokens => 2_800 payment
    // Min maker: 1_500
    // Fees: 300
    let tokens_to_buy = 1_000;
    setup
        .call_fill_order_p2p_by_selling_output(0, TOKEN_IDS[1], 2_800)
        .assert_ok();

    // check balances
    setup
        .b_mock
        .borrow()
        .check_esdt_balance(&setup.treasury, TOKEN_IDS[1], &rust_biguint!(300));

    setup.b_mock.borrow().check_esdt_balance(
        &setup.user,
        TOKEN_IDS[1],
        &rust_biguint!(USER_BALANCE + 1_500),
    );

    setup.b_mock.borrow().check_esdt_balance(
        &setup.taker,
        TOKEN_IDS[0],
        &rust_biguint!(tokens_to_buy),
    );

    // tokens were refunded properly, only 1_800 charged
    setup.b_mock.borrow().check_esdt_balance(
        &setup.taker,
        TOKEN_IDS[1],
        &rust_biguint!(USER_BALANCE - 1_800),
    );
}

#[test]
fn fill_order_by_batch_buying_input_test() {
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

    // send too few tokens
    setup
        .call_fill_order_p2p_by_buying_input(0, 1_000, TOKEN_IDS[1], 500)
        .assert_user_error("Sent too few tokens");

    let tokens_to_buy_1 = 250;
    let tokens_to_buy_2 = 500;
    let tokens_needed_for_p2p_buy_input_1 =
        setup.call_get_tokens_needed_for_p2p_buy_input(0, tokens_to_buy_1);
    let tokens_needed_for_p2p_buy_input_2 =
        setup.call_get_tokens_needed_for_p2p_buy_input(0, tokens_to_buy_2);
    setup
        .call_fill_batch_order(vec![
            (
                0,
                tokens_to_buy_1,
                TOKEN_IDS[1],
                tokens_needed_for_p2p_buy_input_1,
            ),
            (
                0,
                tokens_to_buy_2,
                TOKEN_IDS[1],
                tokens_needed_for_p2p_buy_input_2,
            ),
        ])
        .assert_ok();

    // check balances
    setup
        .b_mock
        .borrow()
        .check_esdt_balance(&setup.treasury, TOKEN_IDS[1], &rust_biguint!(280));

    setup.b_mock.borrow().check_esdt_balance(
        &setup.user,
        TOKEN_IDS[1],
        &rust_biguint!(USER_BALANCE + 1_125),
    );

    setup.b_mock.borrow().check_esdt_balance(
        &setup.taker,
        TOKEN_IDS[0],
        &rust_biguint!(tokens_to_buy_1 + tokens_to_buy_2),
    );

    // check stored order
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
                current_input_amount: managed_biguint!(1_000 - 750),
                min_total_output: managed_biguint!(1_500),
                executor_fee: 1_000,
                creation_timestamp: 0,
                expiration_timestamp: 10 * 60,
            };

            assert_eq!(actual_order, expected_order);
        })
        .assert_ok();
}
