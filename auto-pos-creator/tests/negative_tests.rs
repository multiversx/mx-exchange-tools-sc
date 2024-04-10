#![allow(deprecated)]

use auto_pos_creator::{
    external_sc_interactions::router_actions::SwapOperationType,
    multi_contract_interactions::create_pos_endpoints::CreatePosEndpointsModule,
};
use multiversx_sc::{
    codec::Empty,
    types::{BigUint, ManagedBuffer, MultiValueEncoded},
};
use multiversx_sc_scenario::{
    managed_address, managed_biguint, managed_token_id, rust_biguint,
    whitebox_legacy::TxTokenTransfer, DebugApi,
};
use pos_creator_setup::{PosCreatorSetup, LP_TOKEN_IDS, TOKEN_IDS};
use tests_common::farm_with_locked_rewards_setup::FARM_TOKEN_ID;

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
