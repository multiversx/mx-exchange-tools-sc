#![allow(deprecated)]

use auto_pos_creator::{
    external_sc_interactions::router_actions::SwapOperationType,
    multi_contract_interactions::{
        create_pos_endpoints::CreatePosEndpointsModule, exit_pos_endpoints::ExitPosEndpointsModule,
    },
};
use metastaking_setup::setup_metastaking;
use multiversx_sc::{
    codec::Empty,
    types::{BigUint, ManagedBuffer, MultiValueEncoded},
};
use multiversx_sc_scenario::{
    managed_address, managed_biguint, managed_token_id, rust_biguint,
    whitebox_legacy::TxTokenTransfer, DebugApi,
};
use pos_creator_setup::{PosCreatorSetup, DUAL_YIELD_TOKEN_ID, LP_TOKEN_IDS, TOKEN_IDS};
use sc_whitelist_module::SCWhitelistModule;
use tests_common::{
    farm_staking_setup::STAKING_FARM_TOKEN_ID, farm_with_locked_rewards_setup::FARM_TOKEN_ID,
};

pub mod metastaking_setup;
pub mod pair_setup;
pub mod pos_creator_setup;
pub mod router_setup;

pub static SWAP_TOKENS_FIXED_INPUT_FUNC_NAME: &[u8] = b"swapTokensFixedInput";

#[test]
fn try_create_lp_impossible_swap_path() {
    let pos_creator_setup = PosCreatorSetup::new(
        farm_with_locked_rewards::contract_obj,
        energy_factory::contract_obj,
        pair::contract_obj,
        router::contract_obj,
        farm_staking::contract_obj,
        farm_staking_proxy::contract_obj,
        auto_pos_creator::contract_obj,
    );
    let b_mock = pos_creator_setup.farm_setup.b_mock;

    let user_addr = pos_creator_setup.farm_setup.first_user;
    let user_first_token_balance = 200_000_000u64;
    b_mock.borrow_mut().set_esdt_balance(
        &user_addr,
        TOKEN_IDS[0],
        &rust_biguint!(user_first_token_balance),
    );

    // user enter (B, C) pair with token A
    let first_pair_addr = pos_creator_setup.pair_setups[0]
        .pair_wrapper
        .address_ref()
        .clone();
    let third_pair_addr = pos_creator_setup.pair_setups[2]
        .pair_wrapper
        .address_ref()
        .clone();
    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &user_addr,
            &pos_creator_setup.pos_creator_wrapper,
            TOKEN_IDS[0],
            0,
            &rust_biguint!(user_first_token_balance),
            |sc| {
                let mut swap_operations = MultiValueEncoded::new();
                let swap_operation: SwapOperationType<DebugApi> = (
                    managed_address!(&first_pair_addr),
                    ManagedBuffer::from(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME),
                    managed_token_id!("RAND-123456"),
                    BigUint::from(1u64),
                )
                    .into();
                swap_operations.push(swap_operation);
                let _ = sc.create_lp_pos_from_single_token(
                    managed_address!(&third_pair_addr),
                    1u32.into(),
                    1u32.into(),
                    swap_operations,
                );
            },
        )
        .assert_user_error("Invalid tokens");
}

#[test]
fn try_create_lp_pos_from_same_lp_token() {
    let pos_creator_setup = PosCreatorSetup::new(
        farm_with_locked_rewards::contract_obj,
        energy_factory::contract_obj,
        pair::contract_obj,
        router::contract_obj,
        farm_staking::contract_obj,
        farm_staking_proxy::contract_obj,
        auto_pos_creator::contract_obj,
    );
    let b_mock = pos_creator_setup.farm_setup.b_mock;

    let user_addr = pos_creator_setup.farm_setup.first_user;
    let user_lp_tokens = 142_857_142u64;
    b_mock.borrow_mut().set_esdt_balance(
        &user_addr,
        LP_TOKEN_IDS[2],
        &rust_biguint!(user_lp_tokens),
    );

    // user enter (B, C) pair with token A
    let first_pair_addr = pos_creator_setup.pair_setups[0]
        .pair_wrapper
        .address_ref()
        .clone();
    let third_pair_addr = pos_creator_setup.pair_setups[2]
        .pair_wrapper
        .address_ref()
        .clone();
    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &user_addr,
            &pos_creator_setup.pos_creator_wrapper,
            LP_TOKEN_IDS[2],
            0,
            &rust_biguint!(user_lp_tokens),
            |sc| {
                let mut swap_operations = MultiValueEncoded::new();
                let swap_operation: SwapOperationType<DebugApi> = (
                    managed_address!(&first_pair_addr),
                    ManagedBuffer::from(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME),
                    managed_token_id!(TOKEN_IDS[1]), // Want token B,
                    BigUint::from(1u64),
                )
                    .into();
                swap_operations.push(swap_operation);
                let _ = sc.create_lp_pos_from_single_token(
                    managed_address!(&third_pair_addr),
                    1u32.into(),
                    1u32.into(),
                    swap_operations,
                );
            },
        )
        .assert_user_error("Invalid tokens");
}

#[test]
fn try_create_lp_pos_from_farm_pos() {
    let pos_creator_setup = PosCreatorSetup::new(
        farm_with_locked_rewards::contract_obj,
        energy_factory::contract_obj,
        pair::contract_obj,
        router::contract_obj,
        farm_staking::contract_obj,
        farm_staking_proxy::contract_obj,
        auto_pos_creator::contract_obj,
    );
    let b_mock = pos_creator_setup.farm_setup.b_mock;

    let user_addr = pos_creator_setup.farm_setup.first_user;
    let user_farm_tokens = 142_857_142u64;
    b_mock.borrow_mut().set_nft_balance(
        &user_addr,
        FARM_TOKEN_ID[0],
        1,
        &rust_biguint!(user_farm_tokens),
        &Empty,
    );

    // user enter (B, C) pair with token A
    let first_pair_addr = pos_creator_setup.pair_setups[0]
        .pair_wrapper
        .address_ref()
        .clone();
    let third_pair_addr = pos_creator_setup.pair_setups[2]
        .pair_wrapper
        .address_ref()
        .clone();
    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &user_addr,
            &pos_creator_setup.pos_creator_wrapper,
            FARM_TOKEN_ID[0],
            1,
            &rust_biguint!(user_farm_tokens),
            |sc| {
                let mut swap_operations = MultiValueEncoded::new();
                let swap_operation: SwapOperationType<DebugApi> = (
                    managed_address!(&first_pair_addr),
                    ManagedBuffer::from(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME),
                    managed_token_id!(TOKEN_IDS[1]), // Want token B,
                    BigUint::from(1u64),
                )
                    .into();
                swap_operations.push(swap_operation);
                let _ = sc.create_lp_pos_from_single_token(
                    managed_address!(&third_pair_addr),
                    1u32.into(),
                    1u32.into(),
                    swap_operations,
                );
            },
        )
        .assert_user_error("Only fungible ESDT accepted");
}

#[test]
fn try_create_lp_from_wrong_tokens() {
    let pos_creator_setup = PosCreatorSetup::new(
        farm_with_locked_rewards::contract_obj,
        energy_factory::contract_obj,
        pair::contract_obj,
        router::contract_obj,
        farm_staking::contract_obj,
        farm_staking_proxy::contract_obj,
        auto_pos_creator::contract_obj,
    );
    let b_mock = pos_creator_setup.farm_setup.b_mock;

    let user_addr = pos_creator_setup.farm_setup.first_user;
    let user_first_token_balance = 200_000_000u64;
    let user_second_token_balance = 400_000_000u64;
    b_mock.borrow_mut().set_esdt_balance(
        &user_addr,
        TOKEN_IDS[0],
        &rust_biguint!(user_first_token_balance),
    );
    b_mock.borrow_mut().set_esdt_balance(
        &user_addr,
        TOKEN_IDS[1],
        &rust_biguint!(user_second_token_balance),
    );

    let payments = vec![
        TxTokenTransfer {
            token_identifier: TOKEN_IDS[0].to_vec(),
            nonce: 0,
            value: rust_biguint!(user_first_token_balance),
        },
        TxTokenTransfer {
            token_identifier: TOKEN_IDS[1].to_vec(),
            nonce: 0,
            value: rust_biguint!(user_second_token_balance),
        },
    ];

    let third_pair_addr = pos_creator_setup.pair_setups[2]
        .pair_wrapper
        .address_ref()
        .clone();

    // try enter (B, C) pair with (A, B) tokens
    b_mock
        .borrow_mut()
        .execute_esdt_multi_transfer(
            &user_addr,
            &pos_creator_setup.pos_creator_wrapper,
            &payments,
            |sc| {
                let _ = sc.create_lp_pos_from_two_tokens(
                    managed_address!(&third_pair_addr),
                    managed_biguint!(1),
                    managed_biguint!(1),
                );
            },
        )
        .assert_user_error("Bad payment tokens");
}

#[test]
fn try_create_lp_slippage_error_test() {
    let pos_creator_setup = PosCreatorSetup::new(
        farm_with_locked_rewards::contract_obj,
        energy_factory::contract_obj,
        pair::contract_obj,
        router::contract_obj,
        farm_staking::contract_obj,
        farm_staking_proxy::contract_obj,
        auto_pos_creator::contract_obj,
    );
    let b_mock = pos_creator_setup.farm_setup.b_mock;

    let user_addr = pos_creator_setup.farm_setup.first_user;
    let user_first_token_balance = 200_000_000u64;
    b_mock.borrow_mut().set_esdt_balance(
        &user_addr,
        TOKEN_IDS[0],
        &rust_biguint!(user_first_token_balance),
    );

    // user enter (B, C) pair with token A
    let first_pair_addr = pos_creator_setup.pair_setups[0]
        .pair_wrapper
        .address_ref()
        .clone();
    let third_pair_addr = pos_creator_setup.pair_setups[2]
        .pair_wrapper
        .address_ref()
        .clone();
    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &user_addr,
            &pos_creator_setup.pos_creator_wrapper,
            TOKEN_IDS[0], // Token A
            0,
            &rust_biguint!(user_first_token_balance),
            |sc| {
                // swap_operation -> pair_address, function, token_wanted, amount
                let mut swap_operations = MultiValueEncoded::new();
                let swap_operation: SwapOperationType<DebugApi> = (
                    managed_address!(&first_pair_addr),
                    ManagedBuffer::from(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME),
                    managed_token_id!(TOKEN_IDS[1]), // Want token B
                    BigUint::from(1u64),
                )
                    .into();
                swap_operations.push(swap_operation);
                let _ = sc.create_lp_pos_from_single_token(
                    managed_address!(&third_pair_addr),
                    1u64.into(),
                    2_000_000_001u64.into(),
                    swap_operations,
                );
            },
        )
        .assert_user_error("Insufficient second token computed amount");
}

#[test]
fn try_create_lp_router_swap_slippage_error_test() {
    let pos_creator_setup = PosCreatorSetup::new(
        farm_with_locked_rewards::contract_obj,
        energy_factory::contract_obj,
        pair::contract_obj,
        router::contract_obj,
        farm_staking::contract_obj,
        farm_staking_proxy::contract_obj,
        auto_pos_creator::contract_obj,
    );
    let b_mock = pos_creator_setup.farm_setup.b_mock;

    let user_addr = pos_creator_setup.farm_setup.first_user;
    let user_first_token_balance = 200_000_000u64;
    b_mock.borrow_mut().set_esdt_balance(
        &user_addr,
        TOKEN_IDS[0],
        &rust_biguint!(user_first_token_balance),
    );

    // user enter (B, C) pair with token A
    let first_pair_addr = pos_creator_setup.pair_setups[0]
        .pair_wrapper
        .address_ref()
        .clone();
    let third_pair_addr = pos_creator_setup.pair_setups[2]
        .pair_wrapper
        .address_ref()
        .clone();
    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &user_addr,
            &pos_creator_setup.pos_creator_wrapper,
            TOKEN_IDS[0], // Token A
            0,
            &rust_biguint!(user_first_token_balance),
            |sc| {
                // swap_operation -> pair_address, function, token_wanted, amount
                let mut swap_operations = MultiValueEncoded::new();
                let swap_operation: SwapOperationType<DebugApi> = (
                    managed_address!(&first_pair_addr),
                    ManagedBuffer::from(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME),
                    managed_token_id!(TOKEN_IDS[1]), // Want token B
                    BigUint::from(1_000_000_001u64),
                )
                    .into();
                swap_operations.push(swap_operation);
                let _ = sc.create_lp_pos_from_single_token(
                    managed_address!(&third_pair_addr),
                    1u64.into(),
                    1u64.into(),
                    swap_operations,
                );
            },
        )
        .assert_user_error("Slippage exceeded");
}

#[test]
fn try_create_position_wrong_tokens_test() {
    let pos_creator_setup = PosCreatorSetup::new(
        farm_with_locked_rewards::contract_obj,
        energy_factory::contract_obj,
        pair::contract_obj,
        router::contract_obj,
        farm_staking::contract_obj,
        farm_staking_proxy::contract_obj,
        auto_pos_creator::contract_obj,
    );
    let b_mock = pos_creator_setup.farm_setup.b_mock;

    let user_addr = pos_creator_setup.farm_setup.first_user;
    b_mock
        .borrow_mut()
        .set_esdt_balance(&user_addr, TOKEN_IDS[0], &rust_biguint!(1u64));
    b_mock
        .borrow_mut()
        .set_esdt_balance(&user_addr, TOKEN_IDS[2], &rust_biguint!(1u64));
    let farm_addr = pos_creator_setup.farm_setup.farm_wrappers[0]
        .address_ref()
        .clone();
    let payments = vec![
        TxTokenTransfer {
            token_identifier: TOKEN_IDS[0].to_vec(),
            nonce: 0,
            value: rust_biguint!(1u64),
        },
        TxTokenTransfer {
            token_identifier: TOKEN_IDS[2].to_vec(),
            nonce: 0,
            value: rust_biguint!(1u64),
        },
    ];

    // Try create farm
    b_mock
        .borrow_mut()
        .execute_esdt_multi_transfer(
            &user_addr,
            &pos_creator_setup.pos_creator_wrapper,
            &payments,
            |sc| {
                let _ = sc.create_farm_pos_from_two_tokens(
                    managed_address!(&farm_addr),
                    1u32.into(),
                    1u32.into(),
                );
            },
        )
        .assert_user_error("Bad payment tokens");

    // Try create metastaking
    let ms_addr = pos_creator_setup.ms_wrapper.address_ref().clone();
    b_mock
        .borrow_mut()
        .execute_esdt_multi_transfer(
            &user_addr,
            &pos_creator_setup.pos_creator_wrapper,
            &payments,
            |sc| {
                let _ = sc.create_metastaking_pos_from_two_tokens(
                    managed_address!(&ms_addr),
                    1u32.into(),
                    1u32.into(),
                );
            },
        )
        .assert_user_error("Bad payment tokens");

    // Try create farm staking
    let first_pair_address = pos_creator_setup.pair_setups[0]
        .pair_wrapper
        .address_ref()
        .clone();
    let payment_token_balance = 1_000u64;
    b_mock.borrow_mut().set_esdt_balance(
        &user_addr,
        TOKEN_IDS[0],
        &rust_biguint!(payment_token_balance),
    );
    let fs_addr = pos_creator_setup.fs_wrapper.address_ref().clone();
    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &user_addr,
            &pos_creator_setup.pos_creator_wrapper,
            TOKEN_IDS[0],
            0,
            &rust_biguint!(payment_token_balance),
            |sc| {
                // swap_operation -> pair_address, function, token_wanted, amount
                let mut swap_operations = MultiValueEncoded::new();
                let swap_operation: SwapOperationType<DebugApi> = (
                    managed_address!(&first_pair_address),
                    ManagedBuffer::from(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME),
                    managed_token_id!(TOKEN_IDS[1]), // Want token B
                    BigUint::from(1u64),
                )
                    .into();
                swap_operations.push(swap_operation);
                let _ = sc.create_farm_staking_pos_from_single_token(
                    managed_address!(&fs_addr),
                    1u32.into(),
                    swap_operations,
                );
            },
        )
        .assert_user_error("Invalid swap output token identifier");
}

#[test]
fn try_create_position_wrong_slippage_test() {
    let pos_creator_setup = PosCreatorSetup::new(
        farm_with_locked_rewards::contract_obj,
        energy_factory::contract_obj,
        pair::contract_obj,
        router::contract_obj,
        farm_staking::contract_obj,
        farm_staking_proxy::contract_obj,
        auto_pos_creator::contract_obj,
    );
    let b_mock = pos_creator_setup.farm_setup.b_mock;

    let user_addr = pos_creator_setup.farm_setup.first_user;
    let user_third_token_balance = 600_000_000u64;
    b_mock.borrow_mut().set_esdt_balance(
        &user_addr,
        TOKEN_IDS[2],
        &rust_biguint!(user_third_token_balance),
    );

    // Try create farm position
    let second_pair_addr = pos_creator_setup.pair_setups[1]
        .pair_wrapper
        .address_ref()
        .clone();
    let farm_addr = pos_creator_setup.farm_setup.farm_wrappers[0]
        .address_ref()
        .clone();
    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &user_addr,
            &pos_creator_setup.pos_creator_wrapper,
            TOKEN_IDS[2],
            0,
            &rust_biguint!(user_third_token_balance),
            |sc| {
                // swap_operation -> pair_address, function, token_wanted, amount
                let mut swap_operations = MultiValueEncoded::new();
                let swap_operation: SwapOperationType<DebugApi> = (
                    managed_address!(&second_pair_addr),
                    ManagedBuffer::from(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME),
                    managed_token_id!(TOKEN_IDS[0]), // Want token A
                    BigUint::from(1u64),
                )
                    .into();
                swap_operations.push(swap_operation);
                let _ = sc.create_farm_pos_from_single_token(
                    managed_address!(&farm_addr),
                    1u32.into(),
                    2_000_000_001u64.into(),
                    swap_operations,
                );
            },
        )
        .assert_user_error("Insufficient second token computed amount");

    // Try create metastaking position
    let ms_addr = pos_creator_setup.ms_wrapper.address_ref().clone();
    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &user_addr,
            &pos_creator_setup.pos_creator_wrapper,
            TOKEN_IDS[2], // Token C
            0,
            &rust_biguint!(user_third_token_balance),
            |sc| {
                // swap_operation -> pair_address, function, token_wanted, amount
                let mut swap_operations = MultiValueEncoded::new();
                let swap_operation: SwapOperationType<DebugApi> = (
                    managed_address!(&second_pair_addr),
                    ManagedBuffer::from(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME),
                    managed_token_id!(TOKEN_IDS[0]), // Want token A
                    BigUint::from(1u64),
                )
                    .into();
                swap_operations.push(swap_operation);
                let _ = sc.create_metastaking_pos_from_single_token(
                    managed_address!(&ms_addr),
                    1u32.into(),
                    2_000_000_001u64.into(),
                    swap_operations,
                );
            },
        )
        .assert_user_error("Insufficient second token computed amount");

    // Try create farm staking position
    let fs_addr = pos_creator_setup.fs_wrapper.address_ref().clone();
    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &user_addr,
            &pos_creator_setup.pos_creator_wrapper,
            TOKEN_IDS[2], // Token C
            0,
            &rust_biguint!(user_third_token_balance),
            |sc| {
                // swap_operation -> pair_address, function, token_wanted, amount
                let mut swap_operations = MultiValueEncoded::new();
                let swap_operation: SwapOperationType<DebugApi> = (
                    managed_address!(&second_pair_addr),
                    ManagedBuffer::from(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME),
                    managed_token_id!(TOKEN_IDS[0]), // Want token A
                    BigUint::from(1u64),
                )
                    .into();
                swap_operations.push(swap_operation);
                let _ = sc.create_farm_staking_pos_from_single_token(
                    managed_address!(&fs_addr),
                    2_000_000_001u64.into(),
                    swap_operations,
                );
            },
        )
        .assert_user_error("Slippage exceeded");
}

#[test]
fn try_create_position_wrong_router_swap_slippage_test() {
    let pos_creator_setup = PosCreatorSetup::new(
        farm_with_locked_rewards::contract_obj,
        energy_factory::contract_obj,
        pair::contract_obj,
        router::contract_obj,
        farm_staking::contract_obj,
        farm_staking_proxy::contract_obj,
        auto_pos_creator::contract_obj,
    );
    let b_mock = pos_creator_setup.farm_setup.b_mock;

    let user_addr = pos_creator_setup.farm_setup.first_user;
    let user_third_token_balance = 600_000_000u64;
    b_mock.borrow_mut().set_esdt_balance(
        &user_addr,
        TOKEN_IDS[2],
        &rust_biguint!(user_third_token_balance),
    );

    // Try create farm position
    let second_pair_addr = pos_creator_setup.pair_setups[1]
        .pair_wrapper
        .address_ref()
        .clone();
    let farm_addr = pos_creator_setup.farm_setup.farm_wrappers[0]
        .address_ref()
        .clone();
    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &user_addr,
            &pos_creator_setup.pos_creator_wrapper,
            TOKEN_IDS[2], // Token C
            0,
            &rust_biguint!(user_third_token_balance),
            |sc| {
                // swap_operation -> pair_address, function, token_wanted, amount
                let mut swap_operations = MultiValueEncoded::new();
                let swap_operation: SwapOperationType<DebugApi> = (
                    managed_address!(&second_pair_addr),
                    ManagedBuffer::from(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME),
                    managed_token_id!(TOKEN_IDS[0]), // Want token A
                    BigUint::from(500_000_001u64),
                )
                    .into();
                swap_operations.push(swap_operation);
                let _ = sc.create_farm_pos_from_single_token(
                    managed_address!(&farm_addr),
                    1u32.into(),
                    1u32.into(),
                    swap_operations,
                );
            },
        )
        .assert_user_error("Slippage exceeded");

    // Try create metastaking position
    let ms_addr = pos_creator_setup.ms_wrapper.address_ref().clone();
    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &user_addr,
            &pos_creator_setup.pos_creator_wrapper,
            TOKEN_IDS[2], // Token C
            0,
            &rust_biguint!(user_third_token_balance),
            |sc| {
                // swap_operation -> pair_address, function, token_wanted, amount
                let mut swap_operations = MultiValueEncoded::new();
                let swap_operation: SwapOperationType<DebugApi> = (
                    managed_address!(&second_pair_addr),
                    ManagedBuffer::from(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME),
                    managed_token_id!(TOKEN_IDS[0]), // Want token A
                    BigUint::from(500_000_001u64),
                )
                    .into();
                swap_operations.push(swap_operation);
                let _ = sc.create_metastaking_pos_from_single_token(
                    managed_address!(&ms_addr),
                    1u32.into(),
                    1u32.into(),
                    swap_operations,
                );
            },
        )
        .assert_user_error("Slippage exceeded");

    // Try create farm staking position
    let fs_addr = pos_creator_setup.fs_wrapper.address_ref().clone();
    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &user_addr,
            &pos_creator_setup.pos_creator_wrapper,
            TOKEN_IDS[2], // Token C
            0,
            &rust_biguint!(user_third_token_balance),
            |sc| {
                // swap_operation -> pair_address, function, token_wanted, amount
                let mut swap_operations = MultiValueEncoded::new();
                let swap_operation: SwapOperationType<DebugApi> = (
                    managed_address!(&second_pair_addr),
                    ManagedBuffer::from(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME),
                    managed_token_id!(TOKEN_IDS[0]), // Want token A
                    BigUint::from(500_000_001u64),
                )
                    .into();
                swap_operations.push(swap_operation);
                let _ = sc.create_farm_staking_pos_from_single_token(
                    managed_address!(&fs_addr),
                    1u32.into(),
                    swap_operations,
                );
            },
        )
        .assert_user_error("Slippage exceeded");
}

#[test]
fn try_create_position_wrong_address_test() {
    let pos_creator_setup = PosCreatorSetup::new(
        farm_with_locked_rewards::contract_obj,
        energy_factory::contract_obj,
        pair::contract_obj,
        router::contract_obj,
        farm_staking::contract_obj,
        farm_staking_proxy::contract_obj,
        auto_pos_creator::contract_obj,
    );
    let b_mock = pos_creator_setup.farm_setup.b_mock;

    let user_addr = pos_creator_setup.farm_setup.first_user;
    let user_third_token_balance = 600_000_000u64;
    b_mock.borrow_mut().set_esdt_balance(
        &user_addr,
        TOKEN_IDS[2],
        &rust_biguint!(user_third_token_balance),
    );

    // Try create farm position
    let second_pair_addr = pos_creator_setup.pair_setups[1]
        .pair_wrapper
        .address_ref()
        .clone();
    let wrong_addr = pos_creator_setup.pair_setups[0]
        .pair_wrapper
        .address_ref()
        .clone();
    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &user_addr,
            &pos_creator_setup.pos_creator_wrapper,
            TOKEN_IDS[2], // Token C
            0,
            &rust_biguint!(user_third_token_balance),
            |sc| {
                // swap_operation -> pair_address, function, token_wanted, amount
                let mut swap_operations = MultiValueEncoded::new();
                let swap_operation: SwapOperationType<DebugApi> = (
                    managed_address!(&second_pair_addr),
                    ManagedBuffer::from(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME),
                    managed_token_id!(TOKEN_IDS[0]), // Want token A
                    BigUint::from(1u64),
                )
                    .into();
                swap_operations.push(swap_operation);
                let _ = sc.create_farm_pos_from_single_token(
                    managed_address!(&wrong_addr),
                    1u32.into(),
                    1u32.into(),
                    swap_operations,
                );
            },
        )
        .assert_user_error("storage decode error (key: pair_contract_address): bad array length");

    // Try create metastaking position
    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &user_addr,
            &pos_creator_setup.pos_creator_wrapper,
            TOKEN_IDS[2], // Token C
            0,
            &rust_biguint!(user_third_token_balance),
            |sc| {
                // swap_operation -> pair_address, function, token_wanted, amount
                let mut swap_operations = MultiValueEncoded::new();
                let swap_operation: SwapOperationType<DebugApi> = (
                    managed_address!(&second_pair_addr),
                    ManagedBuffer::from(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME),
                    managed_token_id!(TOKEN_IDS[0]), // Want token A
                    BigUint::from(1u64),
                )
                    .into();
                swap_operations.push(swap_operation);
                let _ = sc.create_farm_pos_from_single_token(
                    managed_address!(&wrong_addr),
                    1u32.into(),
                    1u32.into(),
                    swap_operations,
                );
            },
        )
        .assert_user_error("storage decode error (key: pair_contract_address): bad array length");
}

#[test]
fn try_exit_lp_wrong_address_test() {
    let pos_creator_setup = PosCreatorSetup::new(
        farm_with_locked_rewards::contract_obj,
        energy_factory::contract_obj,
        pair::contract_obj,
        router::contract_obj,
        farm_staking::contract_obj,
        farm_staking_proxy::contract_obj,
        auto_pos_creator::contract_obj,
    );
    let b_mock = pos_creator_setup.farm_setup.b_mock;

    let user_addr = pos_creator_setup.farm_setup.first_user;
    let user_first_token_balance = 200_000_000u64;
    b_mock.borrow_mut().set_esdt_balance(
        &user_addr,
        TOKEN_IDS[0],
        &rust_biguint!(user_first_token_balance),
    );

    // user enter (B, C) pair with token A
    let first_pair_addr = pos_creator_setup.pair_setups[0]
        .pair_wrapper
        .address_ref()
        .clone();
    let third_pair_addr = pos_creator_setup.pair_setups[2]
        .pair_wrapper
        .address_ref()
        .clone();
    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &user_addr,
            &pos_creator_setup.pos_creator_wrapper,
            TOKEN_IDS[0], // Token A
            0,
            &rust_biguint!(user_first_token_balance),
            |sc| {
                // swap_operation -> pair_address, function, token_wanted, amount
                let mut swap_operations = MultiValueEncoded::new();
                let swap_operation: SwapOperationType<DebugApi> = (
                    managed_address!(&first_pair_addr),
                    ManagedBuffer::from(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME),
                    managed_token_id!(TOKEN_IDS[1]), // Want token B
                    BigUint::from(1u64),
                )
                    .into();
                swap_operations.push(swap_operation);
                let _ = sc.create_lp_pos_from_single_token(
                    managed_address!(&third_pair_addr),
                    1u32.into(),
                    1u32.into(),
                    swap_operations,
                );
            },
        )
        .assert_ok();

    // try exit LP pos
    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &user_addr,
            &pos_creator_setup.pos_creator_wrapper,
            LP_TOKEN_IDS[2],
            0,
            &rust_biguint!(1u64),
            |sc| {
                let _ =
                    sc.exit_lp_pos(managed_address!(&first_pair_addr), 1u32.into(), 1u32.into());
            },
        )
        .assert_user_error("Bad payment tokens");
}

#[test]
fn try_exit_farm_wrong_address_test() {
    let pos_creator_setup = PosCreatorSetup::new(
        farm_with_locked_rewards::contract_obj,
        energy_factory::contract_obj,
        pair::contract_obj,
        router::contract_obj,
        farm_staking::contract_obj,
        farm_staking_proxy::contract_obj,
        auto_pos_creator::contract_obj,
    );
    let b_mock = pos_creator_setup.farm_setup.b_mock;

    let user_addr = pos_creator_setup.farm_setup.first_user;
    let user_third_token_balance = 600_000_000u64;
    b_mock.borrow_mut().set_esdt_balance(
        &user_addr,
        TOKEN_IDS[2],
        &rust_biguint!(user_third_token_balance),
    );

    // user enter (A, B) farm with token C
    let second_pair_addr = pos_creator_setup.pair_setups[1]
        .pair_wrapper
        .address_ref()
        .clone();
    let farm_addr = pos_creator_setup.farm_setup.farm_wrappers[0]
        .address_ref()
        .clone();
    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &user_addr,
            &pos_creator_setup.pos_creator_wrapper,
            TOKEN_IDS[2], // Token C
            0,
            &rust_biguint!(user_third_token_balance),
            |sc| {
                // swap_operation -> pair_address, function, token_wanted, amount
                let mut swap_operations = MultiValueEncoded::new();
                let swap_operation: SwapOperationType<DebugApi> = (
                    managed_address!(&second_pair_addr),
                    ManagedBuffer::from(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME),
                    managed_token_id!(TOKEN_IDS[0]), // Want token A
                    BigUint::from(1u64),
                )
                    .into();
                swap_operations.push(swap_operation);
                let _ = sc.create_farm_pos_from_single_token(
                    managed_address!(&farm_addr),
                    1u32.into(),
                    1u32.into(),
                    swap_operations,
                );
            },
        )
        .assert_ok();

    // try exit farm pos
    let wrong_farm_addr = pos_creator_setup.farm_setup.farm_wrappers[1]
        .address_ref()
        .clone();
    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &user_addr,
            &pos_creator_setup.pos_creator_wrapper,
            FARM_TOKEN_ID[0],
            1,
            &rust_biguint!(1u64),
            |sc| {
                let _ =
                    sc.exit_farm_pos(managed_address!(&wrong_farm_addr), 1u32.into(), 1u32.into());
            },
        )
        .assert_user_error("Bad payments");
}

#[test]
fn try_exit_metastaking_wrong_address_test() {
    let pos_creator_setup = PosCreatorSetup::new(
        farm_with_locked_rewards::contract_obj,
        energy_factory::contract_obj,
        pair::contract_obj,
        router::contract_obj,
        farm_staking::contract_obj,
        farm_staking_proxy::contract_obj,
        auto_pos_creator::contract_obj,
    );
    let b_mock = pos_creator_setup.farm_setup.b_mock;

    let user_addr = pos_creator_setup.farm_setup.first_user;
    let user_first_token_balance = 200_000_000u64;
    let user_second_token_balance = 400_000_000u64;
    b_mock.borrow_mut().set_esdt_balance(
        &user_addr,
        TOKEN_IDS[0],
        &rust_biguint!(user_first_token_balance),
    );
    b_mock.borrow_mut().set_esdt_balance(
        &user_addr,
        TOKEN_IDS[1],
        &rust_biguint!(user_second_token_balance),
    );

    let payments = vec![
        TxTokenTransfer {
            token_identifier: TOKEN_IDS[0].to_vec(),
            nonce: 0,
            value: rust_biguint!(user_first_token_balance),
        },
        TxTokenTransfer {
            token_identifier: TOKEN_IDS[1].to_vec(),
            nonce: 0,
            value: rust_biguint!(user_second_token_balance),
        },
    ];

    // user enter (A, B) metastaking farm with (A, B) tokens
    let ms_addr = pos_creator_setup.ms_wrapper.address_ref().clone();
    b_mock
        .borrow_mut()
        .execute_esdt_multi_transfer(
            &user_addr,
            &pos_creator_setup.pos_creator_wrapper,
            &payments,
            |sc| {
                let _ = sc.create_metastaking_pos_from_two_tokens(
                    managed_address!(&ms_addr),
                    1u32.into(),
                    1u32.into(),
                );
            },
        )
        .assert_ok();

    let expected_dual_yield_tokens = 200_000_000u64;
    b_mock.borrow().check_nft_balance::<Empty>(
        &user_addr,
        DUAL_YIELD_TOKEN_ID,
        1,
        &rust_biguint!(expected_dual_yield_tokens),
        None,
    );

    // user exit metastaking pos

    // setup new metastaking
    let ms_wrapper = setup_metastaking(
        &mut b_mock.borrow_mut(),
        farm_staking_proxy::contract_obj,
        &pos_creator_setup.farm_setup.owner,
        pos_creator_setup
            .farm_setup
            .energy_factory_wrapper
            .address_ref(),
        pos_creator_setup.farm_setup.farm_wrappers[0].address_ref(),
        pos_creator_setup.fs_wrapper.address_ref(),
        pos_creator_setup.pair_setups[0].pair_wrapper.address_ref(),
        TOKEN_IDS[1],
        FARM_TOKEN_ID[1],
        STAKING_FARM_TOKEN_ID,
        LP_TOKEN_IDS[1],
        b"DUALYIELD-456789",
    );

    // add auto pos creator SC to metastaking whitelist
    let pos_creator_address = pos_creator_setup.pos_creator_wrapper.address_ref().clone();
    b_mock
        .borrow_mut()
        .execute_tx(
            &pos_creator_setup.farm_setup.owner,
            &ms_wrapper,
            &rust_biguint!(0),
            |sc| {
                sc.sc_whitelist_addresses()
                    .add(&managed_address!(&pos_creator_address));
            },
        )
        .assert_ok();

    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &user_addr,
            &pos_creator_setup.pos_creator_wrapper,
            DUAL_YIELD_TOKEN_ID,
            1,
            &rust_biguint!(expected_dual_yield_tokens),
            |sc| {
                sc.exit_metastaking_pos_endpoint(
                    managed_address!(ms_wrapper.address_ref()),
                    1u32.into(),
                    1u32.into(),
                );
            },
        )
        .assert_user_error("Invalid payment token");
}
