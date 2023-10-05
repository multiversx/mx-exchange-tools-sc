#![allow(deprecated)]

use composable_tasks::compose_tasks::{TaskCall, TaskType};
use composable_tasks_setup::{ComposableTasksSetup, TOKEN_IDS};
use multiversx_sc::types::{ManagedVec, MultiValueEncoded};
use multiversx_sc_scenario::*;
use wegld_swap_setup::WEGLD_TOKEN_ID;

pub mod composable_tasks_setup;
pub mod pair_setup;
pub mod wegld_swap_setup;

#[test]
fn full_composable_tasks_setup_test() {
    let _ = ComposableTasksSetup::new(
        pair::contract_obj,
        multiversx_wegld_swap_sc::contract_obj,
        composable_tasks::contract_obj,
    );
}

#[test]
fn unwrap_single_task_test() {
    let composable_tasks_setup = ComposableTasksSetup::new(
        pair::contract_obj,
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
                let no_args = ManagedVec::new();
                let mut tasks = MultiValueEncoded::new();
                tasks.push((TaskType::UnwrapEGLD, no_args).into());

                sc.compose_tasks(managed_address!(&second_user_addr), tasks);
            },
        )
        .assert_ok();

    b_mock.borrow_mut().check_egld_balance(
        composable_tasks_setup.ct_wrapper.address_ref(),
        &rust_biguint!(user_first_token_balance),
    );
}

#[test]
fn unwrap_send_test() {
    let composable_tasks_setup = ComposableTasksSetup::new(
        pair::contract_obj,
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
                let no_args = ManagedVec::new();
                let mut tasks = MultiValueEncoded::new();
                tasks.push((TaskType::UnwrapEGLD, no_args.clone()).into());
                tasks.push((TaskType::SendEsdt, no_args).into());

                sc.compose_tasks(managed_address!(&second_user_addr), tasks);
            },
        )
        .assert_ok();

    b_mock
        .borrow_mut()
        .check_egld_balance(&second_user_addr, &rust_biguint!(user_first_token_balance));
}

#[test]
fn wrap_send_test() {
    let composable_tasks_setup = ComposableTasksSetup::new(
        pair::contract_obj,
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
                tasks.push((TaskType::WrapEGLD, no_args.clone()).into());
                tasks.push((TaskType::SendEsdt, no_args).into());

                sc.compose_tasks(managed_address!(&second_user_addr), tasks);
            },
        )
        .assert_ok();

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
                let mut swap_args = ManagedVec::new();
                swap_args.push(managed_buffer!(TOKEN_IDS[0]));
                swap_args.push(managed_buffer!(b"1"));

                let mut tasks = MultiValueEncoded::new();
                tasks.push((TaskType::Swap, swap_args).into());

                sc.compose_tasks(managed_address!(&second_user_addr), tasks);
            },
        )
        .assert_ok();

    let expected_balance = 166_666_666u64;
    b_mock.borrow_mut().check_esdt_balance(
        composable_tasks_setup.ct_wrapper.address_ref(),
        TOKEN_IDS[0],
        &rust_biguint!(expected_balance),
    );

    b_mock
        .borrow_mut()
        .check_esdt_balance(&first_user_addr, TOKEN_IDS[0], &rust_biguint!(0));
}


#[test]
fn swap_send_test() {
    let composable_tasks_setup = ComposableTasksSetup::new(
        pair::contract_obj,
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
                let mut swap_args = ManagedVec::new();
                swap_args.push(managed_buffer!(TOKEN_IDS[0]));
                swap_args.push(managed_buffer!(b"1"));

                let no_args = ManagedVec::new();
                let mut tasks = MultiValueEncoded::new();
                tasks.push((TaskType::Swap, swap_args).into());
                tasks.push((TaskType::SendEsdt, no_args).into());

                sc.compose_tasks(managed_address!(&second_user_addr), tasks);
            },
        )
        .assert_ok();

    let expected_balance = 166_666_666u64;
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
        multiversx_wegld_swap_sc::contract_obj,
        composable_tasks::contract_obj,
    );

    let b_mock = composable_tasks_setup.b_mock;
    let first_user_addr = composable_tasks_setup.first_user;
    let second_user_addr = composable_tasks_setup.second_user;

    let user_first_token_balance = 200_000_000u64;

    b_mock.borrow_mut().set_egld_balance(
        &first_user_addr,
        &rust_biguint!(user_first_token_balance),
    );

    b_mock
        .borrow_mut()
        .execute_tx(
            &first_user_addr,
            &composable_tasks_setup.ct_wrapper,
            &rust_biguint!(user_first_token_balance),
            |sc| {
                let mut swap_args = ManagedVec::new();
                swap_args.push(managed_buffer!(TOKEN_IDS[0]));
                swap_args.push(managed_buffer!(b"1"));

                let no_args = ManagedVec::new();
                let mut tasks = MultiValueEncoded::new();

                tasks.push((TaskType::WrapEGLD, no_args.clone()).into());
                tasks.push((TaskType::Swap, swap_args).into());
                tasks.push((TaskType::SendEsdt, no_args).into());

                sc.compose_tasks(managed_address!(&second_user_addr), tasks);
            },
        )
        .assert_ok();

    let expected_balance = 166_666_666u64;
    b_mock.borrow_mut().check_esdt_balance(
        &second_user_addr,
        TOKEN_IDS[0],
        &rust_biguint!(expected_balance),
    );

    b_mock
        .borrow_mut()
        .check_esdt_balance(&first_user_addr, TOKEN_IDS[0], &rust_biguint!(0));
}
