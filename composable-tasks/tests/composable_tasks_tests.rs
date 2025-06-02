#![allow(deprecated)]

use composable_tasks::compose_tasks::{TaskCall, TaskType};
use composable_tasks_setup::{ComposableTasksSetup, TOKEN_IDS};
use multiversx_sc::types::{
    EgldOrEsdtTokenIdentifier, EgldOrEsdtTokenPayment, ManagedVec, MultiValueEncoded,
};
use multiversx_sc_scenario::{whitebox_legacy::TxTokenTransfer, *};
use wegld_swap_setup::{EGLD_TOKEN_ID, WEGLD_TOKEN_ID};

pub mod composable_tasks_setup;
pub mod pair_setup;
pub mod wegld_swap_setup;

pub const SWAP_TOKENS_FIXED_INPUT_FUNC_NAME: &[u8] = b"swapTokensFixedInput";
pub const SWAP_TOKENS_FIXED_OUTPUT_FUNC_NAME: &[u8] = b"swapTokensFixedOutput";

#[test]
fn full_composable_tasks_setup_test() {
    let _ = ComposableTasksSetup::new(
        pair::contract_obj,
        router::contract_obj,
        multiversx_wegld_swap_sc::contract_obj,
        composable_tasks::contract_obj,
    );
}

#[test]
fn unwrap_single_task_test() {
    let composable_tasks_setup = ComposableTasksSetup::new(
        pair::contract_obj,
        router::contract_obj,
        multiversx_wegld_swap_sc::contract_obj,
        composable_tasks::contract_obj,
    );

    let b_mock = composable_tasks_setup.b_mock;
    let first_user_addr = composable_tasks_setup.first_user;

    let user_first_token_balance = 200_000_000u64;

    b_mock.borrow_mut().set_esdt_balance(
        &first_user_addr,
        WEGLD_TOKEN_ID,
        &rust_biguint!(user_first_token_balance),
    );

    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &first_user_addr,
            &composable_tasks_setup.ct_wrapper,
            WEGLD_TOKEN_ID,
            0,
            &rust_biguint!(user_first_token_balance),
            |sc| {
                let no_args = ManagedVec::new();
                let mut tasks = MultiValueEncoded::new();
                tasks.push((TaskType::UnwrapEGLD, no_args).into());

                let expected_token_out = EgldOrEsdtTokenPayment::new(
                    EgldOrEsdtTokenIdentifier::egld(),
                    0,
                    managed_biguint!(user_first_token_balance),
                );

                sc.compose_tasks(expected_token_out, tasks);
            },
        )
        .assert_ok();

    // Funds are sent back to the caller
    b_mock
        .borrow_mut()
        .check_egld_balance(&first_user_addr, &rust_biguint!(user_first_token_balance));
}

#[test]
fn unwrap_send_test() {
    let composable_tasks_setup = ComposableTasksSetup::new(
        pair::contract_obj,
        router::contract_obj,
        multiversx_wegld_swap_sc::contract_obj,
        composable_tasks::contract_obj,
    );

    let b_mock = composable_tasks_setup.b_mock;
    let first_user_addr = composable_tasks_setup.first_user;
    let second_user_addr = composable_tasks_setup.second_user;

    let user_first_token_balance = 200_000_000u64;

    b_mock.borrow_mut().set_esdt_balance(
        &first_user_addr,
        WEGLD_TOKEN_ID,
        &rust_biguint!(user_first_token_balance),
    );

    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &first_user_addr,
            &composable_tasks_setup.ct_wrapper,
            WEGLD_TOKEN_ID,
            0,
            &rust_biguint!(user_first_token_balance),
            |sc| {
                let mut tasks = MultiValueEncoded::new();
                let no_args = ManagedVec::new();
                let mut send_args = ManagedVec::new();
                send_args.push(managed_buffer!(second_user_addr.as_bytes()));

                tasks.push((TaskType::UnwrapEGLD, no_args).into());
                tasks.push((TaskType::SendEgldOrEsdt, send_args).into());

                let expected_token_out = EgldOrEsdtTokenPayment::new(
                    EgldOrEsdtTokenIdentifier::egld(),
                    0,
                    managed_biguint!(user_first_token_balance),
                );

                sc.compose_tasks(expected_token_out, tasks);
            },
        )
        .assert_ok();

    // Funds are sent to the destination
    b_mock
        .borrow_mut()
        .check_egld_balance(&second_user_addr, &rust_biguint!(user_first_token_balance));
}

#[test]
fn wrap_send_test() {
    let composable_tasks_setup = ComposableTasksSetup::new(
        pair::contract_obj,
        router::contract_obj,
        multiversx_wegld_swap_sc::contract_obj,
        composable_tasks::contract_obj,
    );

    let b_mock = composable_tasks_setup.b_mock;
    let first_user_addr = composable_tasks_setup.first_user;
    let second_user_addr = composable_tasks_setup.second_user;

    let user_first_token_balance = 200_000_000u64;

    b_mock
        .borrow_mut()
        .set_egld_balance(&first_user_addr, &rust_biguint!(user_first_token_balance));

    b_mock
        .borrow_mut()
        .execute_tx(
            &first_user_addr,
            &composable_tasks_setup.ct_wrapper,
            &rust_biguint!(user_first_token_balance),
            |sc| {
                let no_args = ManagedVec::new();
                let mut tasks = MultiValueEncoded::new();
                let mut send_args = ManagedVec::new();
                send_args.push(managed_buffer!(second_user_addr.as_bytes()));

                tasks.push((TaskType::WrapEGLD, no_args).into());
                tasks.push((TaskType::SendEgldOrEsdt, send_args).into());

                let expected_token_out = EgldOrEsdtTokenPayment::new(
                    EgldOrEsdtTokenIdentifier::esdt(WEGLD_TOKEN_ID),
                    0,
                    managed_biguint!(user_first_token_balance),
                );

                sc.compose_tasks(expected_token_out, tasks);
            },
        )
        .assert_ok();

    // Funds are sent to the destination
    b_mock.borrow_mut().check_esdt_balance(
        &second_user_addr,
        WEGLD_TOKEN_ID,
        &rust_biguint!(user_first_token_balance),
    );
}

#[test]
fn swap_single_task_test() {
    let composable_tasks_setup = ComposableTasksSetup::new(
        pair::contract_obj,
        router::contract_obj,
        multiversx_wegld_swap_sc::contract_obj,
        composable_tasks::contract_obj,
    );

    let b_mock = composable_tasks_setup.b_mock;
    let first_user_addr = composable_tasks_setup.first_user;

    let user_first_token_balance = 200_000_000u64;

    b_mock.borrow_mut().set_esdt_balance(
        &first_user_addr,
        WEGLD_TOKEN_ID,
        &rust_biguint!(user_first_token_balance),
    );

    let expected_balance = 166_666_666u64;

    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &first_user_addr,
            &composable_tasks_setup.ct_wrapper,
            WEGLD_TOKEN_ID,
            0,
            &rust_biguint!(user_first_token_balance),
            |sc| {
                let mut swap_args = ManagedVec::new();
                swap_args.push(managed_buffer!(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME));
                swap_args.push(managed_buffer!(TOKEN_IDS[0]));
                swap_args.push(managed_buffer!(b"1"));

                let mut tasks = MultiValueEncoded::new();
                tasks.push((TaskType::Swap, swap_args).into());

                let expected_token_out = EgldOrEsdtTokenPayment::new(
                    EgldOrEsdtTokenIdentifier::esdt(TOKEN_IDS[0]),
                    0,
                    managed_biguint!(expected_balance),
                );

                sc.compose_tasks(expected_token_out, tasks);
            },
        )
        .assert_ok();

    // Funds are sent back to the caller
    b_mock.borrow_mut().check_esdt_balance(
        &first_user_addr,
        TOKEN_IDS[0],
        &rust_biguint!(expected_balance),
    );
}

#[test]
fn swap_send_test() {
    let composable_tasks_setup = ComposableTasksSetup::new(
        pair::contract_obj,
        router::contract_obj,
        multiversx_wegld_swap_sc::contract_obj,
        composable_tasks::contract_obj,
    );

    let b_mock = composable_tasks_setup.b_mock;
    let first_user_addr = composable_tasks_setup.first_user;
    let second_user_addr = composable_tasks_setup.second_user;

    let user_first_token_balance = 200_000_000u64;

    b_mock.borrow_mut().set_esdt_balance(
        &first_user_addr,
        WEGLD_TOKEN_ID,
        &rust_biguint!(user_first_token_balance),
    );

    let expected_balance = 166_666_666u64;

    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &first_user_addr,
            &composable_tasks_setup.ct_wrapper,
            WEGLD_TOKEN_ID,
            0,
            &rust_biguint!(user_first_token_balance),
            |sc| {
                let mut swap_args = ManagedVec::new();
                swap_args.push(managed_buffer!(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME));
                swap_args.push(managed_buffer!(TOKEN_IDS[0]));
                swap_args.push(managed_buffer!(b"1"));

                let mut send_args = ManagedVec::new();
                send_args.push(managed_buffer!(second_user_addr.as_bytes()));

                let mut tasks = MultiValueEncoded::new();
                tasks.push((TaskType::Swap, swap_args).into());
                tasks.push((TaskType::SendEgldOrEsdt, send_args).into());

                let expected_token_out = EgldOrEsdtTokenPayment::new(
                    EgldOrEsdtTokenIdentifier::esdt(TOKEN_IDS[0]),
                    0,
                    managed_biguint!(expected_balance),
                );

                sc.compose_tasks(expected_token_out, tasks);
            },
        )
        .assert_ok();

    // Funds are sent to the destination
    b_mock.borrow_mut().check_esdt_balance(
        &second_user_addr,
        TOKEN_IDS[0],
        &rust_biguint!(expected_balance),
    );

    b_mock
        .borrow_mut()
        .check_esdt_balance(&first_user_addr, TOKEN_IDS[0], &rust_biguint!(0));
}

#[test]
fn wrap_swap_send_test() {
    let composable_tasks_setup = ComposableTasksSetup::new(
        pair::contract_obj,
        router::contract_obj,
        multiversx_wegld_swap_sc::contract_obj,
        composable_tasks::contract_obj,
    );

    let b_mock = composable_tasks_setup.b_mock;
    let first_user_addr = composable_tasks_setup.first_user;
    let second_user_addr = composable_tasks_setup.second_user;

    let user_first_token_balance = 200_000_000u64;

    b_mock
        .borrow_mut()
        .set_egld_balance(&first_user_addr, &rust_biguint!(user_first_token_balance));

    let expected_balance = 166_666_666u64;

    b_mock
        .borrow_mut()
        .execute_tx(
            &first_user_addr,
            &composable_tasks_setup.ct_wrapper,
            &rust_biguint!(user_first_token_balance),
            |sc| {
                let mut swap_args = ManagedVec::new();
                swap_args.push(managed_buffer!(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME));
                swap_args.push(managed_buffer!(TOKEN_IDS[0]));
                swap_args.push(managed_buffer!(b"1"));

                let no_args = ManagedVec::new();
                let mut send_args = ManagedVec::new();
                send_args.push(managed_buffer!(second_user_addr.as_bytes()));
                let mut tasks = MultiValueEncoded::new();

                tasks.push((TaskType::WrapEGLD, no_args).into());
                tasks.push((TaskType::Swap, swap_args).into());
                tasks.push((TaskType::SendEgldOrEsdt, send_args).into());

                let expected_token_out = EgldOrEsdtTokenPayment::new(
                    EgldOrEsdtTokenIdentifier::esdt(TOKEN_IDS[0]),
                    0,
                    managed_biguint!(expected_balance),
                );

                sc.compose_tasks(expected_token_out, tasks);
            },
        )
        .assert_ok();

    // Funds are sent to the destination
    b_mock.borrow_mut().check_esdt_balance(
        &second_user_addr,
        TOKEN_IDS[0],
        &rust_biguint!(expected_balance),
    );

    b_mock
        .borrow_mut()
        .check_esdt_balance(&first_user_addr, TOKEN_IDS[0], &rust_biguint!(0));
}

#[test]
fn wrap_swap_fail_test() {
    let composable_tasks_setup = ComposableTasksSetup::new(
        pair::contract_obj,
        router::contract_obj,
        multiversx_wegld_swap_sc::contract_obj,
        composable_tasks::contract_obj,
    );

    let b_mock = composable_tasks_setup.b_mock;
    let first_user_addr = composable_tasks_setup.first_user;
    let second_user_addr = composable_tasks_setup.second_user;

    let user_first_token_balance = 200_000_000u64;

    b_mock
        .borrow_mut()
        .set_egld_balance(&first_user_addr, &rust_biguint!(user_first_token_balance));

    let expected_balance = 200_000_000u64;

    b_mock
        .borrow_mut()
        .execute_tx(
            &first_user_addr,
            &composable_tasks_setup.ct_wrapper,
            &rust_biguint!(user_first_token_balance),
            |sc| {
                let mut swap_args = ManagedVec::new();
                swap_args.push(managed_buffer!(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME));
                swap_args.push(managed_buffer!(TOKEN_IDS[0]));
                swap_args.push(managed_buffer!(b"1"));

                let no_args = ManagedVec::new();
                let mut send_args = ManagedVec::new();
                send_args.push(managed_buffer!(second_user_addr.as_bytes()));
                let mut tasks = MultiValueEncoded::new();

                tasks.push((TaskType::WrapEGLD, no_args).into());
                tasks.push((TaskType::Swap, swap_args).into());
                tasks.push((TaskType::SendEgldOrEsdt, send_args).into());

                let expected_token_out = EgldOrEsdtTokenPayment::new(
                    EgldOrEsdtTokenIdentifier::esdt(TOKEN_IDS[0]),
                    0,
                    managed_biguint!(expected_balance),
                );

                sc.compose_tasks(expected_token_out, tasks);
            },
        )
        .assert_error(
            4,
            "The output token is less or different than the one required by user!",
        );
}

#[test]
fn swap_unwrap_test() {
    let composable_tasks_setup = ComposableTasksSetup::new(
        pair::contract_obj,
        router::contract_obj,
        multiversx_wegld_swap_sc::contract_obj,
        composable_tasks::contract_obj,
    );

    let b_mock = composable_tasks_setup.b_mock;
    let first_user_addr = composable_tasks_setup.first_user;

    let user_first_token_balance = 200_000_000u64;

    b_mock.borrow_mut().set_esdt_balance(
        &first_user_addr,
        TOKEN_IDS[0],
        &rust_biguint!(user_first_token_balance),
    );

    let expected_balance = 166_666_666u64;

    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &first_user_addr,
            &composable_tasks_setup.ct_wrapper,
            TOKEN_IDS[0],
            0,
            &rust_biguint!(user_first_token_balance),
            |sc| {
                let mut swap_args = ManagedVec::new();
                swap_args.push(managed_buffer!(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME));
                swap_args.push(managed_buffer!(TOKEN_IDS[2]));
                swap_args.push(managed_buffer!(b"1"));

                let mut tasks = MultiValueEncoded::new();

                tasks.push((TaskType::Swap, swap_args).into());
                tasks.push((TaskType::UnwrapEGLD, ManagedVec::new()).into());

                let expected_token_out = EgldOrEsdtTokenPayment::new(
                    EgldOrEsdtTokenIdentifier::egld(),
                    0,
                    managed_biguint!(expected_balance),
                );

                sc.compose_tasks(expected_token_out, tasks);
            },
        )
        .assert_ok();

    b_mock
        .borrow_mut()
        .check_egld_balance(&first_user_addr, &rust_biguint!(expected_balance));
}

#[test]
fn swap_unwrap_multiple_input_tokens_test() {
    let composable_tasks_setup = ComposableTasksSetup::new(
        pair::contract_obj,
        router::contract_obj,
        multiversx_wegld_swap_sc::contract_obj,
        composable_tasks::contract_obj,
    );

    let b_mock = composable_tasks_setup.b_mock;
    let first_user_addr = composable_tasks_setup.first_user;

    let user_first_token_balance = 200_000_000u64;
    b_mock.borrow_mut().set_esdt_balance(
        &first_user_addr,
        TOKEN_IDS[0],
        &rust_biguint!(user_first_token_balance),
    );

    let user_first_token_balance = 200_000_000u64;
    b_mock.borrow_mut().set_esdt_balance(
        &first_user_addr,
        TOKEN_IDS[1],
        &rust_biguint!(user_first_token_balance),
    );

    let input_tokens = [
        TxTokenTransfer {
            token_identifier: TOKEN_IDS[0].to_vec(),
            nonce: 0,
            value: rust_biguint!(user_first_token_balance),
        },
        TxTokenTransfer {
            token_identifier: TOKEN_IDS[1].to_vec(),
            nonce: 0,
            value: rust_biguint!(user_first_token_balance),
        },
    ];

    let expected_balance = 166_666_666u64;

    b_mock
        .borrow_mut()
        .execute_esdt_multi_transfer(
            &first_user_addr,
            &composable_tasks_setup.ct_wrapper,
            &input_tokens,
            |sc| {
                let mut swap_args = ManagedVec::new();
                swap_args.push(managed_buffer!(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME));
                swap_args.push(managed_buffer!(WEGLD_TOKEN_ID));
                swap_args.push(managed_buffer!(b"1"));

                let mut tasks = MultiValueEncoded::new();

                tasks.push((TaskType::Swap, swap_args).into());
                tasks.push((TaskType::UnwrapEGLD, ManagedVec::new()).into());

                let expected_token_out = EgldOrEsdtTokenPayment::new(
                    EgldOrEsdtTokenIdentifier::egld(),
                    0,
                    managed_biguint!(expected_balance),
                );

                sc.compose_tasks(expected_token_out, tasks);
            },
        )
        .assert_error(4, "incorrect number of ESDT transfers");
}

#[test]
fn wrap_test() {
    let composable_tasks_setup = ComposableTasksSetup::new(
        pair::contract_obj,
        router::contract_obj,
        multiversx_wegld_swap_sc::contract_obj,
        composable_tasks::contract_obj,
    );

    let b_mock = composable_tasks_setup.b_mock;
    let first_user_addr = composable_tasks_setup.first_user;

    let user_first_token_balance = 200_000_000u64;

    b_mock.borrow_mut().set_esdt_balance(
        &first_user_addr,
        WEGLD_TOKEN_ID,
        &rust_biguint!(user_first_token_balance),
    );
    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &first_user_addr,
            &composable_tasks_setup.ct_wrapper,
            WEGLD_TOKEN_ID,
            0,
            &rust_biguint!(user_first_token_balance),
            |sc| {
                let mut tasks = MultiValueEncoded::new();

                tasks.push((TaskType::WrapEGLD, ManagedVec::new()).into());

                let expected_token_out = EgldOrEsdtTokenPayment::new(
                    EgldOrEsdtTokenIdentifier::esdt(managed_token_id!(WEGLD_TOKEN_ID)),
                    0,
                    managed_biguint!(user_first_token_balance),
                );

                sc.compose_tasks(expected_token_out, tasks);
            },
        )
        .assert_error(4, "Payment token is not EGLD!");

    b_mock
        .borrow_mut()
        .set_egld_balance(&first_user_addr, &rust_biguint!(user_first_token_balance));

    b_mock
        .borrow_mut()
        .execute_tx(
            &first_user_addr,
            &composable_tasks_setup.ct_wrapper,
            &rust_biguint!(user_first_token_balance),
            |sc| {
                let mut tasks = MultiValueEncoded::new();

                tasks.push((TaskType::WrapEGLD, ManagedVec::new()).into());

                let expected_token_out = EgldOrEsdtTokenPayment::new(
                    EgldOrEsdtTokenIdentifier::esdt(managed_token_id!(WEGLD_TOKEN_ID)),
                    0,
                    managed_biguint!(user_first_token_balance),
                );

                sc.compose_tasks(expected_token_out, tasks);
            },
        )
        .assert_ok();
}

#[test]
fn swap_unwrap_wrap_send_test() {
    let composable_tasks_setup = ComposableTasksSetup::new(
        pair::contract_obj,
        router::contract_obj,
        multiversx_wegld_swap_sc::contract_obj,
        composable_tasks::contract_obj,
    );

    let b_mock = composable_tasks_setup.b_mock;
    let first_user_addr = composable_tasks_setup.first_user;
    let second_user_addr = composable_tasks_setup.second_user;

    let user_first_token_balance = 200_000_000u64;

    b_mock.borrow_mut().set_esdt_balance(
        &first_user_addr,
        TOKEN_IDS[0],
        &rust_biguint!(user_first_token_balance),
    );
    let expected_balance = 166_666_666u64;

    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &first_user_addr,
            &composable_tasks_setup.ct_wrapper,
            TOKEN_IDS[0],
            0,
            &rust_biguint!(user_first_token_balance),
            |sc| {
                let mut swap_args = ManagedVec::new();
                swap_args.push(managed_buffer!(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME));
                swap_args.push(managed_buffer!(WEGLD_TOKEN_ID));
                swap_args.push(managed_buffer!(b"1"));

                let mut tasks = MultiValueEncoded::new();

                tasks.push((TaskType::Swap, swap_args).into());
                tasks.push((TaskType::UnwrapEGLD, ManagedVec::new()).into());
                tasks.push((TaskType::WrapEGLD, ManagedVec::new()).into());

                let mut send_args = ManagedVec::new();
                send_args.push(managed_buffer!(second_user_addr.as_bytes()));
                tasks.push((TaskType::SendEgldOrEsdt, send_args).into());

                let expected_token_out = EgldOrEsdtTokenPayment::new(
                    EgldOrEsdtTokenIdentifier::esdt(managed_token_id!(WEGLD_TOKEN_ID)),
                    0,
                    managed_biguint!(expected_balance),
                );

                sc.compose_tasks(expected_token_out, tasks);
            },
        )
        .assert_ok();

    b_mock.borrow_mut().check_esdt_balance(
        &second_user_addr,
        WEGLD_TOKEN_ID,
        &rust_biguint!(expected_balance),
    );
}

#[test]
fn wrap_swap_input_swap_output_unwrap_send_test() {
    let composable_tasks_setup = ComposableTasksSetup::new(
        pair::contract_obj,
        router::contract_obj,
        multiversx_wegld_swap_sc::contract_obj,
        composable_tasks::contract_obj,
    );

    let b_mock = composable_tasks_setup.b_mock;
    let first_user_addr = composable_tasks_setup.first_user;
    let second_user_addr = composable_tasks_setup.second_user;
    let second_pair_addr = composable_tasks_setup.pair_setups[1]
        .pair_wrapper
        .address_ref();

    let user_first_token_balance = 200_000_010u64;

    b_mock
        .borrow_mut()
        .set_egld_balance(&first_user_addr, &rust_biguint!(user_first_token_balance));
    let expected_balance = 166_666_666u64;

    b_mock
        .borrow_mut()
        .execute_tx(
            &first_user_addr,
            &composable_tasks_setup.ct_wrapper,
            &rust_biguint!(user_first_token_balance),
            |sc| {
                let mut tasks = MultiValueEncoded::new();
                tasks.push((TaskType::WrapEGLD, ManagedVec::new()).into());

                let mut router_swap_args = ManagedVec::new();
                router_swap_args.push(managed_buffer!(second_pair_addr.as_bytes()));
                router_swap_args.push(managed_buffer!(SWAP_TOKENS_FIXED_OUTPUT_FUNC_NAME));
                router_swap_args.push(managed_buffer!(TOKEN_IDS[0]));
                router_swap_args.push(managed_buffer!(
                    &rust_biguint!(expected_balance).to_bytes_be()
                ));
                router_swap_args.push(managed_buffer!(second_pair_addr.as_bytes()));
                router_swap_args.push(managed_buffer!(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME));
                router_swap_args.push(managed_buffer!(WEGLD_TOKEN_ID));
                router_swap_args.push(managed_buffer!(b"1"));

                tasks.push((TaskType::RouterSwap, router_swap_args).into());
                tasks.push((TaskType::UnwrapEGLD, ManagedVec::new()).into());

                let mut send_args = ManagedVec::new();
                send_args.push(managed_buffer!(second_user_addr.as_bytes()));
                tasks.push((TaskType::SendEgldOrEsdt, send_args).into());

                let expected_token_out = EgldOrEsdtTokenPayment::new(
                    EgldOrEsdtTokenIdentifier::egld(),
                    0,
                    managed_biguint!(expected_balance),
                );

                sc.compose_tasks(expected_token_out, tasks);
            },
        )
        .assert_ok();

    b_mock
        .borrow_mut()
        .check_egld_balance(&second_user_addr, &rust_biguint!(199_999_999u64));

    b_mock
        .borrow_mut()
        .check_esdt_balance(&second_user_addr, TOKEN_IDS[0], &rust_biguint!(0u64));

    b_mock.borrow_mut().check_esdt_balance(
        &second_user_addr,
        WEGLD_TOKEN_ID,
        &rust_biguint!(10u64),
    );
}

#[test]
fn wrap_swap_tokens_fixed_output_test() {
    let composable_tasks_setup = ComposableTasksSetup::new(
        pair::contract_obj,
        router::contract_obj,
        multiversx_wegld_swap_sc::contract_obj,
        composable_tasks::contract_obj,
    );

    let b_mock = composable_tasks_setup.b_mock;
    let first_user_addr = composable_tasks_setup.first_user;
    let second_user_addr = composable_tasks_setup.second_user;

    let user_first_token_balance = 200_000_001u64;

    b_mock
        .borrow_mut()
        .set_egld_balance(&first_user_addr, &rust_biguint!(user_first_token_balance));
    let expected_balance = 166_666_666u64;

    b_mock
        .borrow_mut()
        .execute_tx(
            &first_user_addr,
            &composable_tasks_setup.ct_wrapper,
            &rust_biguint!(user_first_token_balance),
            |sc| {
                let mut swap_args = ManagedVec::new();
                swap_args.push(managed_buffer!(SWAP_TOKENS_FIXED_OUTPUT_FUNC_NAME));
                swap_args.push(managed_buffer!(TOKEN_IDS[0]));
                swap_args.push(managed_buffer!(
                    &rust_biguint!(expected_balance).to_bytes_be()
                ));

                let mut tasks = MultiValueEncoded::new();

                tasks.push((TaskType::WrapEGLD, ManagedVec::new()).into());
                tasks.push((TaskType::Swap, swap_args).into());

                let mut send_args = ManagedVec::new();
                send_args.push(managed_buffer!(second_user_addr.as_bytes()));
                tasks.push((TaskType::SendEgldOrEsdt, send_args).into());

                let expected_token_out = EgldOrEsdtTokenPayment::new(
                    EgldOrEsdtTokenIdentifier::esdt(managed_token_id!(TOKEN_IDS[0])),
                    0,
                    managed_biguint!(expected_balance),
                );

                sc.compose_tasks(expected_token_out, tasks);
            },
        )
        .assert_ok();

    // rest of the swap (166_666_666 swapped to 200_000_000 and 1 remaining)
    b_mock
        .borrow_mut()
        .check_esdt_balance(&second_user_addr, WEGLD_TOKEN_ID, &rust_biguint!(1u64));
}

#[test]
fn send_intermediary_step_test() {
    let composable_tasks_setup = ComposableTasksSetup::new(
        pair::contract_obj,
        router::contract_obj,
        multiversx_wegld_swap_sc::contract_obj,
        composable_tasks::contract_obj,
    );

    let b_mock = composable_tasks_setup.b_mock;
    let first_user_addr = composable_tasks_setup.first_user;
    let second_user_addr = composable_tasks_setup.second_user;

    let user_first_token_balance = 200_000_001u64;

    b_mock.borrow_mut().set_esdt_balance(
        &first_user_addr,
        TOKEN_IDS[0],
        &rust_biguint!(user_first_token_balance),
    );

    let expected_balance = 166_666_666u64;

    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &first_user_addr,
            &composable_tasks_setup.ct_wrapper,
            TOKEN_IDS[0],
            0,
            &rust_biguint!(user_first_token_balance),
            |sc| {
                let mut swap_args = ManagedVec::new();
                swap_args.push(managed_buffer!(SWAP_TOKENS_FIXED_OUTPUT_FUNC_NAME));
                swap_args.push(managed_buffer!(WEGLD_TOKEN_ID));
                swap_args.push(managed_buffer!(
                    &rust_biguint!(expected_balance).to_bytes_be()
                ));

                let mut send_args = ManagedVec::new();
                send_args.push(managed_buffer!(second_user_addr.as_bytes()));

                let mut tasks = MultiValueEncoded::new();
                tasks.push((TaskType::Swap, swap_args).into());
                tasks.push((TaskType::SendEgldOrEsdt, send_args).into());
                tasks.push((TaskType::UnwrapEGLD, ManagedVec::new()).into());

                let expected_token_out = EgldOrEsdtTokenPayment::new(
                    EgldOrEsdtTokenIdentifier::esdt(WEGLD_TOKEN_ID),
                    0,
                    managed_biguint!(expected_balance),
                );

                sc.compose_tasks(expected_token_out, tasks);
            },
        )
        .assert_ok();

    // Funds are sent to the destination
    b_mock.borrow_mut().check_esdt_balance(
        &second_user_addr,
        WEGLD_TOKEN_ID,
        &rust_biguint!(expected_balance),
    );
    b_mock
        .borrow_mut()
        .check_esdt_balance(&second_user_addr, TOKEN_IDS[0], &rust_biguint!(1));

    // The unwrap step was not executed -> no EGLD
    b_mock
        .borrow_mut()
        .check_egld_balance(&second_user_addr, &rust_biguint!(0));

    // No tokens received by the caller
    b_mock
        .borrow_mut()
        .check_esdt_balance(&first_user_addr, TOKEN_IDS[0], &rust_biguint!(0));
    b_mock
        .borrow_mut()
        .check_esdt_balance(&first_user_addr, EGLD_TOKEN_ID, &rust_biguint!(0));
    b_mock
        .borrow_mut()
        .check_egld_balance(&first_user_addr, &rust_biguint!(0));
}

// This test is commented for now but will enable it later
fn _wrap_swap_tokens_fixed_output_exact_amount_test() {
    let composable_tasks_setup = ComposableTasksSetup::new(
        pair::contract_obj,
        router::contract_obj,
        multiversx_wegld_swap_sc::contract_obj,
        composable_tasks::contract_obj,
    );

    let b_mock = composable_tasks_setup.b_mock;
    let first_user_addr = composable_tasks_setup.first_user;
    let second_user_addr = composable_tasks_setup.second_user;

    let user_first_token_balance = 200_000_000u64;

    b_mock
        .borrow_mut()
        .set_egld_balance(&first_user_addr, &rust_biguint!(user_first_token_balance));
    let expected_balance = 166_666_666u64;

    b_mock
        .borrow_mut()
        .execute_tx(
            &first_user_addr,
            &composable_tasks_setup.ct_wrapper,
            &rust_biguint!(user_first_token_balance),
            |sc| {
                let mut swap_args = ManagedVec::new();
                swap_args.push(managed_buffer!(SWAP_TOKENS_FIXED_OUTPUT_FUNC_NAME));
                swap_args.push(managed_buffer!(TOKEN_IDS[0]));
                swap_args.push(managed_buffer!(
                    &rust_biguint!(expected_balance).to_bytes_be()
                ));

                let mut tasks = MultiValueEncoded::new();

                tasks.push((TaskType::WrapEGLD, ManagedVec::new()).into());
                tasks.push((TaskType::Swap, swap_args).into());

                let mut send_args = ManagedVec::new();
                send_args.push(managed_buffer!(second_user_addr.as_bytes()));
                tasks.push((TaskType::SendEgldOrEsdt, send_args).into());

                let expected_token_out = EgldOrEsdtTokenPayment::new(
                    EgldOrEsdtTokenIdentifier::esdt(managed_token_id!(TOKEN_IDS[0])),
                    0,
                    managed_biguint!(expected_balance),
                );

                sc.compose_tasks(expected_token_out, tasks);
            },
        )
        .assert_ok();

    // rest of the swap (166_666_666 swapped to 200_000_000 and 1 remaining)
    b_mock
        .borrow_mut()
        .check_esdt_balance(&second_user_addr, WEGLD_TOKEN_ID, &rust_biguint!(1u64));
}

#[test]
fn swap_tokens_fixed_output_unwrap_test() {
    let composable_tasks_setup = ComposableTasksSetup::new(
        pair::contract_obj,
        router::contract_obj,
        multiversx_wegld_swap_sc::contract_obj,
        composable_tasks::contract_obj,
    );

    let b_mock = composable_tasks_setup.b_mock;
    let first_user_addr = composable_tasks_setup.first_user;
    let second_user_addr = composable_tasks_setup.second_user;

    let user_first_token_balance = 200_000_001u64;

    b_mock.borrow_mut().set_esdt_balance(
        &first_user_addr,
        TOKEN_IDS[0],
        &rust_biguint!(user_first_token_balance),
    );
    let expected_balance = 166_666_666u64;

    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &first_user_addr,
            &composable_tasks_setup.ct_wrapper,
            TOKEN_IDS[0],
            0,
            &rust_biguint!(user_first_token_balance),
            |sc| {
                let mut swap_args = ManagedVec::new();
                swap_args.push(managed_buffer!(SWAP_TOKENS_FIXED_OUTPUT_FUNC_NAME));
                swap_args.push(managed_buffer!(WEGLD_TOKEN_ID));
                swap_args.push(managed_buffer!(
                    &rust_biguint!(expected_balance).to_bytes_be()
                ));

                let mut tasks = MultiValueEncoded::new();

                tasks.push((TaskType::Swap, swap_args).into());
                tasks.push((TaskType::UnwrapEGLD, ManagedVec::new()).into());

                let mut send_args = ManagedVec::new();
                send_args.push(managed_buffer!(second_user_addr.as_bytes()));
                tasks.push((TaskType::SendEgldOrEsdt, send_args).into());

                let expected_token_out = EgldOrEsdtTokenPayment::new(
                    EgldOrEsdtTokenIdentifier::egld(),
                    0,
                    managed_biguint!(expected_balance),
                );

                sc.compose_tasks(expected_token_out, tasks);
            },
        )
        .assert_ok();

    // rest of the input token
    b_mock
        .borrow_mut()
        .check_esdt_balance(&second_user_addr, TOKEN_IDS[0], &rust_biguint!(1u64));
}

#[test]
fn multiple_swap_tokens_fixed_output_test() {
    let composable_tasks_setup = ComposableTasksSetup::new(
        pair::contract_obj,
        router::contract_obj,
        multiversx_wegld_swap_sc::contract_obj,
        composable_tasks::contract_obj,
    );

    let b_mock = composable_tasks_setup.b_mock;
    let first_user_addr = composable_tasks_setup.first_user;
    let user_first_token_balance = 1_000_000_004u64;

    b_mock.borrow_mut().set_esdt_balance(
        &first_user_addr,
        TOKEN_IDS[0],
        &rust_biguint!(user_first_token_balance),
    );
    let expected_balance_token1 = 1_000_000_001u64;
    let expected_balance_token2 = 1_000_000_000u64;
    let expected_balance_token3 = 1u64;

    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &first_user_addr,
            &composable_tasks_setup.ct_wrapper,
            TOKEN_IDS[0],
            0,
            &rust_biguint!(user_first_token_balance),
            |sc| {
                let mut tasks = MultiValueEncoded::new();

                let mut swap_args1 = ManagedVec::new();
                swap_args1.push(managed_buffer!(SWAP_TOKENS_FIXED_OUTPUT_FUNC_NAME));
                swap_args1.push(managed_buffer!(TOKEN_IDS[1]));
                swap_args1.push(managed_buffer!(
                    &rust_biguint!(expected_balance_token1).to_bytes_be()
                ));
                tasks.push((TaskType::Swap, swap_args1).into());

                let mut swap_args2 = ManagedVec::new();
                swap_args2.push(managed_buffer!(SWAP_TOKENS_FIXED_OUTPUT_FUNC_NAME));
                swap_args2.push(managed_buffer!(WEGLD_TOKEN_ID));
                swap_args2.push(managed_buffer!(
                    &rust_biguint!(expected_balance_token2).to_bytes_be()
                ));
                tasks.push((TaskType::Swap, swap_args2).into());

                let mut swap_args3 = ManagedVec::new();
                swap_args3.push(managed_buffer!(SWAP_TOKENS_FIXED_OUTPUT_FUNC_NAME));
                swap_args3.push(managed_buffer!(TOKEN_IDS[3]));
                swap_args3.push(managed_buffer!(
                    &rust_biguint!(expected_balance_token3).to_bytes_be()
                ));
                tasks.push((TaskType::Swap, swap_args3).into());

                let expected_token_out = EgldOrEsdtTokenPayment::new(
                    EgldOrEsdtTokenIdentifier::esdt(managed_token_id!(TOKEN_IDS[3])),
                    0,
                    managed_biguint!(expected_balance_token3),
                );

                sc.compose_tasks(expected_token_out, tasks);
            },
        )
        .assert_ok();

    // rest of the input token[0]
    b_mock
        .borrow_mut()
        .check_esdt_balance(&first_user_addr, TOKEN_IDS[0], &rust_biguint!(1u64));

    // rest of the input token[1]
    b_mock.borrow_mut().check_esdt_balance(
        &first_user_addr,
        TOKEN_IDS[1],
        &rust_biguint!(500_000_000u64),
    );

    // rest of the input token[2]
    b_mock.borrow_mut().check_esdt_balance(
        &first_user_addr,
        WEGLD_TOKEN_ID,
        &rust_biguint!(999_999_998u64),
    );

    // rest of the input token[3]
    b_mock
        .borrow_mut()
        .check_esdt_balance(&first_user_addr, TOKEN_IDS[3], &rust_biguint!(1u64));
}

///////// ROUTER ////////////
#[test]
fn swap_router_single_task_test() {
    let composable_tasks_setup = ComposableTasksSetup::new(
        pair::contract_obj,
        router::contract_obj,
        multiversx_wegld_swap_sc::contract_obj,
        composable_tasks::contract_obj,
    );

    let b_mock = composable_tasks_setup.b_mock;
    let first_user_addr = composable_tasks_setup.first_user;

    let second_pair_addr = composable_tasks_setup.pair_setups[1]
        .pair_wrapper
        .address_ref();

    let user_first_token_balance = 200_000_000u64;

    b_mock.borrow_mut().set_esdt_balance(
        &first_user_addr,
        WEGLD_TOKEN_ID,
        &rust_biguint!(user_first_token_balance),
    );

    let expected_balance = 166_666_666u64;

    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &first_user_addr,
            &composable_tasks_setup.ct_wrapper,
            WEGLD_TOKEN_ID,
            0,
            &rust_biguint!(user_first_token_balance),
            |sc| {
                let mut swap_args = ManagedVec::new();
                swap_args.push(managed_buffer!(second_pair_addr.as_bytes()));
                swap_args.push(managed_buffer!(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME));
                swap_args.push(managed_buffer!(TOKEN_IDS[0]));
                swap_args.push(managed_buffer!(
                    &rust_biguint!(expected_balance).to_bytes_be()
                ));

                let mut tasks = MultiValueEncoded::new();
                tasks.push((TaskType::RouterSwap, swap_args).into());
                let expected_token_out = EgldOrEsdtTokenPayment::new(
                    EgldOrEsdtTokenIdentifier::esdt(managed_token_id!(TOKEN_IDS[0])),
                    0,
                    managed_biguint!(expected_balance),
                );

                sc.compose_tasks(expected_token_out, tasks);
            },
        )
        .assert_ok();

    // Funds are sent back to the caller
    b_mock.borrow_mut().check_esdt_balance(
        &first_user_addr,
        TOKEN_IDS[0],
        &rust_biguint!(expected_balance),
    );
}

#[test]
fn swap_token_fixed_output_router_test() {
    let composable_tasks_setup = ComposableTasksSetup::new(
        pair::contract_obj,
        router::contract_obj,
        multiversx_wegld_swap_sc::contract_obj,
        composable_tasks::contract_obj,
    );

    let b_mock = composable_tasks_setup.b_mock;
    let first_user_addr = composable_tasks_setup.first_user;

    let second_pair_addr = composable_tasks_setup.pair_setups[1]
        .pair_wrapper
        .address_ref();

    let user_first_token_balance = 200_000_001u64;

    b_mock.borrow_mut().set_esdt_balance(
        &first_user_addr,
        WEGLD_TOKEN_ID,
        &rust_biguint!(user_first_token_balance),
    );

    let expected_balance = 166_666_666u64;

    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &first_user_addr,
            &composable_tasks_setup.ct_wrapper,
            WEGLD_TOKEN_ID,
            0,
            &rust_biguint!(user_first_token_balance),
            |sc| {
                let mut swap_args = ManagedVec::new();
                swap_args.push(managed_buffer!(second_pair_addr.as_bytes()));
                swap_args.push(managed_buffer!(SWAP_TOKENS_FIXED_OUTPUT_FUNC_NAME));
                swap_args.push(managed_buffer!(TOKEN_IDS[0]));
                swap_args.push(managed_buffer!(
                    &rust_biguint!(expected_balance).to_bytes_be()
                ));

                let mut tasks = MultiValueEncoded::new();
                tasks.push((TaskType::RouterSwap, swap_args).into());
                let expected_token_out = EgldOrEsdtTokenPayment::new(
                    EgldOrEsdtTokenIdentifier::esdt(managed_token_id!(TOKEN_IDS[0])),
                    0,
                    managed_biguint!(expected_balance),
                );

                sc.compose_tasks(expected_token_out, tasks);
            },
        )
        .assert_ok();

    // Funds are sent back to the caller
    b_mock.borrow_mut().check_esdt_balance(
        &first_user_addr,
        TOKEN_IDS[0],
        &rust_biguint!(expected_balance),
    );
}

#[test]
fn multiple_swap_token_multiple_fixed_output_router_test() {
    let composable_tasks_setup = ComposableTasksSetup::new(
        pair::contract_obj,
        router::contract_obj,
        multiversx_wegld_swap_sc::contract_obj,
        composable_tasks::contract_obj,
    );

    let b_mock = composable_tasks_setup.b_mock;
    let first_user_addr = composable_tasks_setup.first_user;

    let first_pair_addr = composable_tasks_setup.pair_setups[0]
        .pair_wrapper
        .address_ref();

    let second_pair_addr = composable_tasks_setup.pair_setups[1]
        .pair_wrapper
        .address_ref();

    let user_first_token_balance = 200_000_001u64;

    b_mock.borrow_mut().set_esdt_balance(
        &first_user_addr,
        WEGLD_TOKEN_ID,
        &rust_biguint!(user_first_token_balance),
    );

    let amount_tokens_out_tokens_0 = 166_666_666u64;
    let amount_tokens_out_tokens_1 = 10_000u64;

    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &first_user_addr,
            &composable_tasks_setup.ct_wrapper,
            WEGLD_TOKEN_ID,
            0,
            &rust_biguint!(user_first_token_balance),
            |sc| {
                let mut swap_args = ManagedVec::new();
                swap_args.push(managed_buffer!(second_pair_addr.as_bytes()));
                swap_args.push(managed_buffer!(SWAP_TOKENS_FIXED_OUTPUT_FUNC_NAME));
                swap_args.push(managed_buffer!(TOKEN_IDS[0]));
                swap_args.push(managed_buffer!(
                    &rust_biguint!(amount_tokens_out_tokens_0).to_bytes_be()
                ));

                swap_args.push(managed_buffer!(first_pair_addr.as_bytes()));
                swap_args.push(managed_buffer!(SWAP_TOKENS_FIXED_OUTPUT_FUNC_NAME));
                swap_args.push(managed_buffer!(TOKEN_IDS[1]));
                swap_args.push(managed_buffer!(
                    &rust_biguint!(amount_tokens_out_tokens_1).to_bytes_be()
                ));

                let mut tasks = MultiValueEncoded::new();
                tasks.push((TaskType::RouterSwap, swap_args).into());
                let expected_token_out = EgldOrEsdtTokenPayment::new(
                    EgldOrEsdtTokenIdentifier::esdt(managed_token_id!(TOKEN_IDS[1])),
                    0,
                    managed_biguint!(amount_tokens_out_tokens_1),
                );

                sc.compose_tasks(expected_token_out, tasks);
            },
        )
        .assert_ok();

    let expected_balance_tokens_0 = 166_661_665u64;
    let expected_balance_tokens_2 = 1u64;

    // Funds are sent back to the caller
    b_mock.borrow_mut().check_esdt_balance(
        &first_user_addr,
        TOKEN_IDS[0],
        &rust_biguint!(expected_balance_tokens_0),
    );
    b_mock.borrow_mut().check_esdt_balance(
        &first_user_addr,
        TOKEN_IDS[1],
        &rust_biguint!(amount_tokens_out_tokens_1),
    );
    b_mock.borrow_mut().check_esdt_balance(
        &first_user_addr,
        TOKEN_IDS[2],
        &rust_biguint!(expected_balance_tokens_2),
    );
}

#[test]
fn swap_router_send_task_test() {
    let composable_tasks_setup = ComposableTasksSetup::new(
        pair::contract_obj,
        router::contract_obj,
        multiversx_wegld_swap_sc::contract_obj,
        composable_tasks::contract_obj,
    );

    let b_mock = composable_tasks_setup.b_mock;
    let first_user_addr = composable_tasks_setup.first_user;
    let second_user_addr = composable_tasks_setup.second_user;

    let second_pair_addr = composable_tasks_setup.pair_setups[1]
        .pair_wrapper
        .address_ref();

    let user_first_token_balance = 200_000_000u64;

    b_mock.borrow_mut().set_esdt_balance(
        &first_user_addr,
        WEGLD_TOKEN_ID,
        &rust_biguint!(user_first_token_balance),
    );

    let expected_balance = 166_666_666u64;

    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &first_user_addr,
            &composable_tasks_setup.ct_wrapper,
            WEGLD_TOKEN_ID,
            0,
            &rust_biguint!(user_first_token_balance),
            |sc| {
                let mut swap_args = ManagedVec::new();
                swap_args.push(managed_buffer!(second_pair_addr.as_bytes()));
                swap_args.push(managed_buffer!(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME));
                swap_args.push(managed_buffer!(TOKEN_IDS[0]));
                swap_args.push(managed_buffer!(
                    &rust_biguint!(expected_balance).to_bytes_be()
                ));

                let mut tasks = MultiValueEncoded::new();
                tasks.push((TaskType::RouterSwap, swap_args).into());

                let mut send_args = ManagedVec::new();
                send_args.push(managed_buffer!(second_user_addr.as_bytes()));

                tasks.push((TaskType::SendEgldOrEsdt, send_args).into());

                let expected_token_out = EgldOrEsdtTokenPayment::new(
                    EgldOrEsdtTokenIdentifier::esdt(managed_token_id!(TOKEN_IDS[0])),
                    0,
                    managed_biguint!(expected_balance),
                );
                sc.compose_tasks(expected_token_out, tasks);
            },
        )
        .assert_ok();

    // Funds are sent back to the caller
    b_mock.borrow_mut().check_esdt_balance(
        &second_user_addr,
        TOKEN_IDS[0],
        &rust_biguint!(expected_balance),
    );
}
