use auto_farm::common::unique_payments::UniquePayments;
use dca_module::user_data::{
    action::{
        action_types::{ActionInfo, TradeFrequency, HOURLY_TIMESTAMP},
        storage::ActionStorageModule,
    },
    funds::FundsModule,
};
use multiversx_sc::types::{EsdtTokenPayment, ManagedVec};
use multiversx_sc_scenario::{
    imports::TxTokenTransfer, managed_biguint, managed_token_id, rust_biguint,
};

use crate::dca_module_setup::{DcaModuleSetup, DummySwapArgs, START_TIME, TOKEN_IDS};

// partially stolen from pos creator
pub mod dca_module_setup;

// stolen from pos creator
pub mod pair_setup;
pub mod router_setup;

#[test]
fn setup_test() {
    let _ = DcaModuleSetup::new(
        pair::contract_obj,
        router::contract_obj,
        dca_module::contract_obj,
    );
}

#[test]
fn user_deposit_ok_test() {
    let setup = DcaModuleSetup::new(
        pair::contract_obj,
        router::contract_obj,
        dca_module::contract_obj,
    );

    let transfers = &[TxTokenTransfer {
        token_identifier: TOKEN_IDS[0].to_vec(),
        nonce: 0,
        value: rust_biguint!(1_000),
    }];
    setup.user_deposit(transfers).assert_ok();

    setup
        .b_mock
        .borrow_mut()
        .execute_query(&setup.dca_module_wrapper, |sc| {
            let actual_user_funds = sc.user_funds(1).get();
            let expected_payments_vec = ManagedVec::from_single_item(EsdtTokenPayment::new(
                managed_token_id!(TOKEN_IDS[0]),
                0,
                managed_biguint!(1_000),
            ));

            assert_eq!(
                actual_user_funds,
                UniquePayments::new_from_payments(expected_payments_vec)
            )
        })
        .assert_ok();

    // user deposit same token again
    setup.user_deposit(transfers).assert_ok();

    setup
        .b_mock
        .borrow_mut()
        .execute_query(&setup.dca_module_wrapper, |sc| {
            let actual_user_funds = sc.user_funds(1).get();
            let expected_payments_vec = ManagedVec::from_single_item(EsdtTokenPayment::new(
                managed_token_id!(TOKEN_IDS[0]),
                0,
                managed_biguint!(2_000),
            ));

            assert_eq!(
                actual_user_funds,
                UniquePayments::new_from_payments(expected_payments_vec)
            )
        })
        .assert_ok();

    setup
        .b_mock
        .borrow()
        .check_esdt_balance(&setup.user, TOKEN_IDS[0], &rust_biguint!(3_000));
    setup.b_mock.borrow().check_esdt_balance(
        setup.dca_module_wrapper.address_ref(),
        TOKEN_IDS[0],
        &rust_biguint!(2_000),
    );
}

#[test]
fn withdraw_part_test() {
    let setup = DcaModuleSetup::new(
        pair::contract_obj,
        router::contract_obj,
        dca_module::contract_obj,
    );

    let transfers = &[TxTokenTransfer {
        token_identifier: TOKEN_IDS[0].to_vec(),
        nonce: 0,
        value: rust_biguint!(1_000),
    }];
    setup.user_deposit(transfers).assert_ok();

    let to_withdraw = &[TxTokenTransfer {
        token_identifier: TOKEN_IDS[0].to_vec(),
        nonce: 0,
        value: rust_biguint!(700),
    }];
    setup.user_withdraw_part(to_withdraw).assert_ok();

    setup
        .b_mock
        .borrow_mut()
        .execute_query(&setup.dca_module_wrapper, |sc| {
            let actual_user_funds = sc.user_funds(1).get();
            let expected_payments_vec = ManagedVec::from_single_item(EsdtTokenPayment::new(
                managed_token_id!(TOKEN_IDS[0]),
                0,
                managed_biguint!(300),
            ));

            assert_eq!(
                actual_user_funds,
                UniquePayments::new_from_payments(expected_payments_vec)
            )
        })
        .assert_ok();

    setup
        .b_mock
        .borrow()
        .check_esdt_balance(&setup.user, TOKEN_IDS[0], &rust_biguint!(4_700));
    setup.b_mock.borrow().check_esdt_balance(
        setup.dca_module_wrapper.address_ref(),
        TOKEN_IDS[0],
        &rust_biguint!(300),
    );
}

#[test]
fn withdraw_all_test() {
    let setup = DcaModuleSetup::new(
        pair::contract_obj,
        router::contract_obj,
        dca_module::contract_obj,
    );

    let transfers = &[TxTokenTransfer {
        token_identifier: TOKEN_IDS[0].to_vec(),
        nonce: 0,
        value: rust_biguint!(1_000),
    }];
    setup.user_deposit(transfers).assert_ok();

    setup.user_withdraw_all().assert_ok();

    setup
        .b_mock
        .borrow_mut()
        .execute_query(&setup.dca_module_wrapper, |sc| {
            assert!(sc.user_funds(1).is_empty());
        })
        .assert_ok();

    setup
        .b_mock
        .borrow()
        .check_esdt_balance(&setup.user, TOKEN_IDS[0], &rust_biguint!(5_000));
    setup.b_mock.borrow().check_esdt_balance(
        setup.dca_module_wrapper.address_ref(),
        TOKEN_IDS[0],
        &rust_biguint!(0),
    );
}

#[test]
fn register_action_ok_test() {
    let setup = DcaModuleSetup::new(
        pair::contract_obj,
        router::contract_obj,
        dca_module::contract_obj,
    );

    let transfers = &[TxTokenTransfer {
        token_identifier: TOKEN_IDS[0].to_vec(),
        nonce: 0,
        value: rust_biguint!(1_000),
    }];
    setup.user_deposit(transfers).assert_ok();

    // action A -> C
    setup
        .register_action(TradeFrequency::Hourly, 2, TOKEN_IDS[0], 500, TOKEN_IDS[2])
        .assert_ok();

    setup
        .b_mock
        .borrow_mut()
        .execute_query(&setup.dca_module_wrapper, |sc| {
            let actual_action_info = sc.action_info(1).get();
            assert_eq!(
                actual_action_info,
                ActionInfo {
                    owner_id: 1,
                    trade_frequency: TradeFrequency::Hourly,
                    input_token_id: managed_token_id!(TOKEN_IDS[0]),
                    input_tokens_amount: managed_biguint!(500),
                    output_token_id: managed_token_id!(TOKEN_IDS[2]),
                    last_action_timestamp: 0,
                    total_actions_left: 2,
                    action_in_progress: false,
                }
            )
        })
        .assert_ok();
}

#[test]
fn execute_action_direct_path_ok_test() {
    let mut setup = DcaModuleSetup::new(
        pair::contract_obj,
        router::contract_obj,
        dca_module::contract_obj,
    );

    let transfers = &[TxTokenTransfer {
        token_identifier: TOKEN_IDS[0].to_vec(),
        nonce: 0,
        value: rust_biguint!(1_000),
    }];
    setup.user_deposit(transfers).assert_ok();

    // action A -> C
    setup
        .register_action(TradeFrequency::Hourly, 2, TOKEN_IDS[0], 500, TOKEN_IDS[2])
        .assert_ok();

    // direct path of A -> C
    let swap_args = vec![DummySwapArgs {
        pair_addr: setup.pair_setups[1].pair_wrapper.address_ref().clone(),
        requested_token: TOKEN_IDS[2].to_vec(),
        min_amount_out: 1,
    }];
    setup.execute_action(1, swap_args.clone()).assert_ok();

    setup
        .b_mock
        .borrow_mut()
        .execute_query(&setup.dca_module_wrapper, |sc| {
            let actual_action_info = sc.action_info(1).get();
            assert_eq!(
                actual_action_info,
                ActionInfo {
                    owner_id: 1,
                    trade_frequency: TradeFrequency::Hourly,
                    input_token_id: managed_token_id!(TOKEN_IDS[0]),
                    input_tokens_amount: managed_biguint!(500),
                    output_token_id: managed_token_id!(TOKEN_IDS[2]),
                    last_action_timestamp: START_TIME,
                    total_actions_left: 1,
                    action_in_progress: false,
                }
            );

            let remaining_payment =
                EsdtTokenPayment::new(managed_token_id!(TOKEN_IDS[0]), 0, managed_biguint!(500));
            assert_eq!(
                sc.user_funds(1).get(),
                UniquePayments::new_from_payments(ManagedVec::from_single_item(remaining_payment))
            );
        })
        .assert_ok();

    // 500 * 6, minus some fees
    setup
        .b_mock
        .borrow()
        .check_esdt_balance(&setup.user, TOKEN_IDS[2], &rust_biguint!(2_999));

    // try execute action too early
    setup
        .execute_action(1, swap_args.clone())
        .assert_user_error("Trying to execute action too early");

    setup.advance_time(HOURLY_TIMESTAMP);

    // execute action second time ok
    setup.execute_action(1, swap_args.clone()).assert_ok();

    // action was cleared, as it was the last one
    setup
        .b_mock
        .borrow_mut()
        .execute_query(&setup.dca_module_wrapper, |sc| {
            assert!(sc.action_info(1).is_empty())
        })
        .assert_ok();

    // try execute action - no storage
    setup
        .execute_action(1, swap_args)
        .assert_user_error("Action either already executed or doesn't exist");
}

#[test]
fn execute_action_complex_path_ok_test() {
    let setup = DcaModuleSetup::new(
        pair::contract_obj,
        router::contract_obj,
        dca_module::contract_obj,
    );

    let transfers = &[TxTokenTransfer {
        token_identifier: TOKEN_IDS[0].to_vec(),
        nonce: 0,
        value: rust_biguint!(1_000),
    }];
    setup.user_deposit(transfers).assert_ok();

    // action A -> C
    setup
        .register_action(TradeFrequency::Hourly, 2, TOKEN_IDS[0], 500, TOKEN_IDS[2])
        .assert_ok();

    // complex path A -> B, B -> C
    let swap_args = vec![
        DummySwapArgs {
            pair_addr: setup.pair_setups[0].pair_wrapper.address_ref().clone(),
            requested_token: TOKEN_IDS[1].to_vec(),
            min_amount_out: 1,
        },
        DummySwapArgs {
            pair_addr: setup.pair_setups[2].pair_wrapper.address_ref().clone(),
            requested_token: TOKEN_IDS[2].to_vec(),
            min_amount_out: 1,
        },
    ];
    setup.execute_action(1, swap_args.clone()).assert_ok();

    setup
        .b_mock
        .borrow_mut()
        .execute_query(&setup.dca_module_wrapper, |sc| {
            let actual_action_info = sc.action_info(1).get();
            assert_eq!(
                actual_action_info,
                ActionInfo {
                    owner_id: 1,
                    trade_frequency: TradeFrequency::Hourly,
                    input_token_id: managed_token_id!(TOKEN_IDS[0]),
                    input_tokens_amount: managed_biguint!(500),
                    output_token_id: managed_token_id!(TOKEN_IDS[2]),
                    last_action_timestamp: START_TIME,
                    total_actions_left: 1,
                    action_in_progress: false,
                }
            );

            let remaining_payment =
                EsdtTokenPayment::new(managed_token_id!(TOKEN_IDS[0]), 0, managed_biguint!(500));
            assert_eq!(
                sc.user_funds(1).get(),
                UniquePayments::new_from_payments(ManagedVec::from_single_item(remaining_payment))
            );
        })
        .assert_ok();

    // First, A -> B gives ~1000 tokens, then B -> C gives ~3000 tokens, minus some fees
    setup
        .b_mock
        .borrow()
        .check_esdt_balance(&setup.user, TOKEN_IDS[2], &rust_biguint!(2_996));
}

#[test]
fn execute_action_failed_test() {
    let setup = DcaModuleSetup::new(
        pair::contract_obj,
        router::contract_obj,
        dca_module::contract_obj,
    );

    let transfers = &[TxTokenTransfer {
        token_identifier: TOKEN_IDS[0].to_vec(),
        nonce: 0,
        value: rust_biguint!(1_000),
    }];
    setup.user_deposit(transfers).assert_ok();

    // action A -> C
    setup
        .register_action(TradeFrequency::Hourly, 2, TOKEN_IDS[0], 500, TOKEN_IDS[2])
        .assert_ok();

    // direct path of A -> C, but wrong pair
    let swap_args = vec![DummySwapArgs {
        pair_addr: setup.pair_setups[2].pair_wrapper.address_ref().clone(),
        requested_token: TOKEN_IDS[2].to_vec(),
        min_amount_out: 1,
    }];
    setup.execute_action(1, swap_args.clone()).assert_ok();

    setup
        .b_mock
        .borrow_mut()
        .execute_query(&setup.dca_module_wrapper, |sc| {
            // action info didn't change
            let actual_action_info = sc.action_info(1).get();
            assert_eq!(
                actual_action_info,
                ActionInfo {
                    owner_id: 1,
                    trade_frequency: TradeFrequency::Hourly,
                    input_token_id: managed_token_id!(TOKEN_IDS[0]),
                    input_tokens_amount: managed_biguint!(500),
                    output_token_id: managed_token_id!(TOKEN_IDS[2]),
                    last_action_timestamp: 0,
                    total_actions_left: 2,
                    action_in_progress: false,
                }
            );

            // user funds were still deducted
            let remaining_payment =
                EsdtTokenPayment::new(managed_token_id!(TOKEN_IDS[0]), 0, managed_biguint!(500));
            assert_eq!(
                sc.user_funds(1).get(),
                UniquePayments::new_from_payments(ManagedVec::from_single_item(remaining_payment))
            );

            // but nr retries were updated
            assert_eq!(sc.nr_retries_per_action(1).get(), 1);
        })
        .assert_ok();

    // try action again
    setup.execute_action(1, swap_args).assert_ok();

    setup
        .b_mock
        .borrow_mut()
        .execute_query(&setup.dca_module_wrapper, |sc| {
            // action was cleared, as it failed too many times
            assert!(sc.action_info(1).is_empty());
            assert!(sc.nr_retries_per_action(1).is_empty());
            assert_eq!(sc.user_funds(1).get(), UniquePayments::new());
        })
        .assert_ok();

    // user did get all his tokens back
    setup
        .b_mock
        .borrow()
        .check_esdt_balance(&setup.user, TOKEN_IDS[0], &rust_biguint!(5_000));
}
