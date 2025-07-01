use auto_farm::common::unique_payments::UniquePayments;
use dca_module::user_data::funds::FundsModule;
use multiversx_sc::types::{EsdtTokenPayment, ManagedVec};
use multiversx_sc_scenario::{
    imports::TxTokenTransfer, managed_biguint, managed_token_id, rust_biguint,
};

use crate::dca_module_setup::{DcaModuleSetup, TOKEN_IDS};

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
