#![allow(deprecated)]

use auto_pos_creator::{
    external_sc_interactions::router_actions::SwapOperationType,
    multi_contract_interactions::{
        create_pos_endpoints::CreatePosEndpointsModule, exit_pos_endpoints::ExitPosEndpointsModule,
    },
};
use farm::exit_penalty::ExitPenaltyModule;
use farm_staking::token_attributes::{StakingFarmTokenAttributes, UnbondSftAttributes};
use metastaking_setup::DUAL_YIELD_TOKEN_ID;
use multiversx_sc::{
    codec::Empty,
    types::{BigUint, ManagedAddress, ManagedBuffer, MultiValueEncoded},
};
use multiversx_sc_scenario::{
    managed_address, managed_biguint, managed_token_id, rust_biguint,
    testing_framework::TxTokenTransfer, DebugApi,
};
use pos_creator_setup::{PosCreatorSetup, LP_TOKEN_IDS, TOKEN_IDS};
use tests_common::{
    farm_staking_setup::STAKING_FARM_TOKEN_ID,
    farm_with_locked_rewards_setup::{FARM_TOKEN_ID, LOCKED_REWARD_TOKEN_ID},
};

pub mod metastaking_setup;
pub mod pair_setup;
pub mod pos_creator_setup;
pub mod router_setup;

pub const SWAP_TOKENS_FIXED_INPUT_FUNC_NAME: &[u8] = b"swapTokensFixedInput";

#[test]
fn full_pos_creator_setup_test() {
    let _ = PosCreatorSetup::new(
        farm_with_locked_rewards::contract_obj,
        energy_factory::contract_obj,
        pair::contract_obj,
        router::contract_obj,
        farm_staking::contract_obj,
        farm_staking_proxy::contract_obj,
        auto_pos_creator::contract_obj,
    );
}

#[test]
fn enter_lp_through_pos_creator_test() {
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

    // User adds liquidity in pool B-C, using token A
    // Route: All tokens A are swapped to token B
    // Half of the swap output is then swapped to token C
    // Add liquidity using the resulted tokens B and C
    let expected_remaining_third_token = 61_224_488u64;
    let expected_lp_token = 142_857_142u64;

    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, TOKEN_IDS[0], &rust_biguint!(0));
    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, TOKEN_IDS[1], &rust_biguint!(0));
    // Should have tokens remaining after add liquidity
    b_mock.borrow().check_esdt_balance(
        &user_addr,
        TOKEN_IDS[2],
        &rust_biguint!(expected_remaining_third_token),
    );
    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, LP_TOKEN_IDS[0], &rust_biguint!(0));
    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, LP_TOKEN_IDS[1], &rust_biguint!(0));
    b_mock.borrow().check_esdt_balance(
        &user_addr,
        LP_TOKEN_IDS[2],
        &rust_biguint!(expected_lp_token),
    );

    // Check auto pos creator balance (should be 0 for all tokens)
    b_mock.borrow().check_esdt_balance(
        pos_creator_setup.pos_creator_wrapper.address_ref(),
        TOKEN_IDS[0],
        &rust_biguint!(0),
    );
    b_mock.borrow().check_esdt_balance(
        pos_creator_setup.pos_creator_wrapper.address_ref(),
        TOKEN_IDS[1],
        &rust_biguint!(0),
    );
    b_mock.borrow().check_esdt_balance(
        pos_creator_setup.pos_creator_wrapper.address_ref(),
        TOKEN_IDS[2],
        &rust_biguint!(0),
    );
    b_mock.borrow().check_esdt_balance(
        pos_creator_setup.pos_creator_wrapper.address_ref(),
        LP_TOKEN_IDS[2],
        &rust_biguint!(0),
    );

    // exit LP pos
    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &user_addr,
            &pos_creator_setup.pos_creator_wrapper,
            LP_TOKEN_IDS[2],
            0,
            &rust_biguint!(expected_lp_token),
            |sc| {
                let _ =
                    sc.exit_lp_pos(managed_address!(&third_pair_addr), 1u32.into(), 1u32.into());
            },
        )
        .assert_ok();

    let expected_second_token_amount_from_lp = 166_666_665u64;
    let expected_third_token_amount_from_lp = 367_346_937u64;

    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, TOKEN_IDS[0], &rust_biguint!(0));
    b_mock.borrow().check_esdt_balance(
        &user_addr,
        TOKEN_IDS[1],
        &rust_biguint!(expected_second_token_amount_from_lp),
    );
    b_mock.borrow().check_esdt_balance(
        &user_addr,
        TOKEN_IDS[2],
        &rust_biguint!(expected_third_token_amount_from_lp + expected_remaining_third_token),
    );
    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, LP_TOKEN_IDS[2], &rust_biguint!(0));
}

#[test]
fn enter_lp_with_swap_to_second_token_of_the_pair_test() {
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
    let user_first_token_balance = 600_000_000u64;
    b_mock.borrow_mut().set_esdt_balance(
        &user_addr,
        TOKEN_IDS[2],
        &rust_biguint!(user_first_token_balance),
    );

    // user enter (A, B) pair with token C
    // Route: token C -> full swap to token B (second token) -> half swap to token A -> add LP to (A, B)
    let expected_remaining_first_token = 1_600_000u64;
    let expected_lp_tokens = 39_999_998u64;
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
            TOKEN_IDS[2], // Token C
            0,
            &rust_biguint!(user_first_token_balance),
            |sc| {
                // swap_operation -> pair_address, function, token_wanted, amount
                let mut swap_operations = MultiValueEncoded::new();
                let swap_operation: SwapOperationType<DebugApi> = (
                    managed_address!(&third_pair_addr), // Swap tokens in LP (B, C)
                    ManagedBuffer::from(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME),
                    managed_token_id!(TOKEN_IDS[1]), // Want token B
                    BigUint::from(1u64),
                )
                    .into();
                swap_operations.push(swap_operation);
                let output_payments = sc.create_lp_pos_from_single_token(
                    managed_address!(&first_pair_addr), // Add LP to (A, B)
                    1u32.into(),
                    1u32.into(),
                    swap_operations,
                );

                // Should have 2 output payments (dust remaing tokens and LP tokens)
                assert_eq!(output_payments.len(), 2);
                assert_eq!(
                    output_payments.get(0).token_identifier,
                    managed_token_id!(TOKEN_IDS[0])
                );
                assert_eq!(
                    output_payments.get(0).amount,
                    managed_biguint!(expected_remaining_first_token)
                );
                assert_eq!(
                    output_payments.get(1).token_identifier,
                    managed_token_id!(LP_TOKEN_IDS[0])
                );
                assert_eq!(
                    output_payments.get(1).amount,
                    managed_biguint!(expected_lp_tokens)
                );
            },
        )
        .assert_ok();

    // Check user balance
    b_mock.borrow().check_esdt_balance(
        &user_addr,
        TOKEN_IDS[0],
        &rust_biguint!(expected_remaining_first_token),
    );
    b_mock.borrow().check_esdt_balance(
        &user_addr,
        LP_TOKEN_IDS[0],
        &rust_biguint!(expected_lp_tokens),
    );
}

#[test]
fn enter_lp_and_farm_through_pos_creator() {
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

    // User adds liquidity in pool (A, B), using token C
    // Route: All tokens C are swapped to token A
    // Half of the swap output is then swapped to token B
    // Add liquidity using the resulted tokens A and B
    let expected_remaining_second_token = 3_780_718u64;
    let expected_farm_token = 43_478_260u64;

    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, TOKEN_IDS[0], &rust_biguint!(0));
    // Should have tokens remaining after add liquidity
    b_mock.borrow().check_esdt_balance(
        &user_addr,
        TOKEN_IDS[1],
        &rust_biguint!(expected_remaining_second_token),
    );
    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, TOKEN_IDS[2], &rust_biguint!(0));
    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, LP_TOKEN_IDS[0], &rust_biguint!(0));
    b_mock.borrow().check_nft_balance::<Empty>(
        &user_addr,
        FARM_TOKEN_ID[0],
        1,
        &rust_biguint!(expected_farm_token),
        None,
    );

    // Check auto pos creator balance (should be 0 for all tokens)
    b_mock.borrow().check_esdt_balance(
        pos_creator_setup.pos_creator_wrapper.address_ref(),
        TOKEN_IDS[0],
        &rust_biguint!(0),
    );
    b_mock.borrow().check_esdt_balance(
        pos_creator_setup.pos_creator_wrapper.address_ref(),
        TOKEN_IDS[1],
        &rust_biguint!(0),
    );
    b_mock.borrow().check_esdt_balance(
        pos_creator_setup.pos_creator_wrapper.address_ref(),
        TOKEN_IDS[2],
        &rust_biguint!(0),
    );
    b_mock.borrow().check_esdt_balance(
        pos_creator_setup.pos_creator_wrapper.address_ref(),
        LP_TOKEN_IDS[0],
        &rust_biguint!(0),
    );
    b_mock.borrow().check_nft_balance::<Empty>(
        pos_creator_setup.pos_creator_wrapper.address_ref(),
        FARM_TOKEN_ID[0],
        1,
        &rust_biguint!(0),
        None,
    );

    // exit LP pos
    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &user_addr,
            &pos_creator_setup.pos_creator_wrapper,
            FARM_TOKEN_ID[0],
            1,
            &rust_biguint!(expected_farm_token),
            |sc| {
                let _ = sc.exit_farm_pos(managed_address!(&farm_addr), 1u32.into(), 1u32.into());
            },
        )
        .assert_ok();

    let expected_first_token_amount_from_lp = 45_454_544u64;
    let expected_second_token_amount_from_lp = 83_175_801u64;

    b_mock.borrow().check_esdt_balance(
        &user_addr,
        TOKEN_IDS[0],
        &rust_biguint!(expected_first_token_amount_from_lp),
    );
    b_mock.borrow().check_esdt_balance(
        &user_addr,
        TOKEN_IDS[1],
        &rust_biguint!(expected_second_token_amount_from_lp + expected_remaining_second_token),
    );
    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, TOKEN_IDS[2], &rust_biguint!(0));
    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, LP_TOKEN_IDS[0], &rust_biguint!(0));
    b_mock.borrow().check_nft_balance::<Empty>(
        &user_addr,
        FARM_TOKEN_ID[0],
        1,
        &rust_biguint!(0),
        None,
    );
    // Check auto pos creator balance (should be 0 for all tokens)
    b_mock.borrow().check_esdt_balance(
        pos_creator_setup.pos_creator_wrapper.address_ref(),
        TOKEN_IDS[0],
        &rust_biguint!(0),
    );
    b_mock.borrow().check_esdt_balance(
        pos_creator_setup.pos_creator_wrapper.address_ref(),
        TOKEN_IDS[1],
        &rust_biguint!(0),
    );
    b_mock.borrow().check_esdt_balance(
        pos_creator_setup.pos_creator_wrapper.address_ref(),
        TOKEN_IDS[2],
        &rust_biguint!(0),
    );
    b_mock.borrow().check_esdt_balance(
        pos_creator_setup.pos_creator_wrapper.address_ref(),
        LP_TOKEN_IDS[0],
        &rust_biguint!(0),
    );
    b_mock.borrow().check_nft_balance::<Empty>(
        pos_creator_setup.pos_creator_wrapper.address_ref(),
        FARM_TOKEN_ID[0],
        1,
        &rust_biguint!(0),
        None,
    );
}

#[test]
fn enter_lp_farm_and_metastaking_through_pos_creator_test() {
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

    // User enters metastaking by adding liquidity in pool (A, B) -> farm (A, B) -> metastaking (A, B)
    // Route: All tokens C are swapped to token A in pool (A, C)
    // Half of the swap output is then swapped to token B
    // Add liquidity using the resulted tokens A and B
    let second_pair_addr = pos_creator_setup.pair_setups[1]
        .pair_wrapper
        .address_ref()
        .clone();
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
                    1u32.into(),
                    swap_operations,
                );
            },
        )
        .assert_ok();

    let expected_remaining_second_token = 3_780_718u64;
    let expected_dual_yield_token = 41_666_665u64;

    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, TOKEN_IDS[0], &rust_biguint!(0));
    // Should have tokens remaining after add liquidity
    b_mock.borrow().check_esdt_balance(
        &user_addr,
        TOKEN_IDS[1],
        &rust_biguint!(expected_remaining_second_token),
    );
    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, TOKEN_IDS[2], &rust_biguint!(0));
    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, LP_TOKEN_IDS[0], &rust_biguint!(0));
    b_mock.borrow().check_nft_balance::<Empty>(
        &user_addr,
        FARM_TOKEN_ID[0],
        1,
        &rust_biguint!(0),
        None,
    );
    b_mock.borrow().check_nft_balance::<Empty>(
        &user_addr,
        DUAL_YIELD_TOKEN_ID,
        1,
        &rust_biguint!(expected_dual_yield_token),
        None,
    );

    // Check auto pos creator balance (should be 0 for all tokens)
    b_mock.borrow().check_esdt_balance(
        pos_creator_setup.pos_creator_wrapper.address_ref(),
        TOKEN_IDS[0],
        &rust_biguint!(0),
    );
    b_mock.borrow().check_esdt_balance(
        pos_creator_setup.pos_creator_wrapper.address_ref(),
        TOKEN_IDS[1],
        &rust_biguint!(0),
    );
    b_mock.borrow().check_esdt_balance(
        pos_creator_setup.pos_creator_wrapper.address_ref(),
        TOKEN_IDS[2],
        &rust_biguint!(0),
    );
    b_mock.borrow().check_esdt_balance(
        pos_creator_setup.pos_creator_wrapper.address_ref(),
        LP_TOKEN_IDS[0],
        &rust_biguint!(0),
    );
    b_mock.borrow().check_nft_balance::<Empty>(
        pos_creator_setup.pos_creator_wrapper.address_ref(),
        FARM_TOKEN_ID[0],
        1,
        &rust_biguint!(0),
        None,
    );
    b_mock.borrow().check_nft_balance::<Empty>(
        pos_creator_setup.pos_creator_wrapper.address_ref(),
        DUAL_YIELD_TOKEN_ID,
        1,
        &rust_biguint!(0),
        None,
    );

    // exit metastaking pos
    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &user_addr,
            &pos_creator_setup.pos_creator_wrapper,
            DUAL_YIELD_TOKEN_ID,
            1,
            &rust_biguint!(expected_dual_yield_token),
            |sc| {
                let _ = sc.exit_metastaking_pos_endpoint(
                    managed_address!(&ms_addr),
                    1u32.into(),
                    1u32.into(),
                );
            },
        )
        .assert_ok();

    let expected_second_token_amount_from_lp = 83_175_801u64;
    let expected_staking_farm_token_amount = 45_454_544u64;

    // Passes through unbond contract, so balance should be 0 until tokens are unbonded
    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, TOKEN_IDS[0], &rust_biguint!(0));
    b_mock.borrow().check_esdt_balance(
        &user_addr,
        TOKEN_IDS[1],
        &rust_biguint!(expected_second_token_amount_from_lp + expected_remaining_second_token),
    );
    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, TOKEN_IDS[2], &rust_biguint!(0));
    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, LP_TOKEN_IDS[0], &rust_biguint!(0));
    b_mock.borrow().check_nft_balance::<Empty>(
        &user_addr,
        FARM_TOKEN_ID[0],
        1,
        &rust_biguint!(0),
        None,
    );
    b_mock.borrow().check_nft_balance::<Empty>(
        &user_addr,
        STAKING_FARM_TOKEN_ID,
        2,
        &rust_biguint!(expected_staking_farm_token_amount),
        None,
    );

    // Check auto pos creator balance (should be 0 for all tokens)
    b_mock.borrow().check_esdt_balance(
        pos_creator_setup.pos_creator_wrapper.address_ref(),
        TOKEN_IDS[0],
        &rust_biguint!(0),
    );
    b_mock.borrow().check_esdt_balance(
        pos_creator_setup.pos_creator_wrapper.address_ref(),
        TOKEN_IDS[1],
        &rust_biguint!(0),
    );
    b_mock.borrow().check_esdt_balance(
        pos_creator_setup.pos_creator_wrapper.address_ref(),
        TOKEN_IDS[2],
        &rust_biguint!(0),
    );
    b_mock.borrow().check_esdt_balance(
        pos_creator_setup.pos_creator_wrapper.address_ref(),
        LP_TOKEN_IDS[0],
        &rust_biguint!(0),
    );
    b_mock.borrow().check_nft_balance::<Empty>(
        pos_creator_setup.pos_creator_wrapper.address_ref(),
        FARM_TOKEN_ID[0],
        1,
        &rust_biguint!(0),
        None,
    );
    b_mock.borrow().check_nft_balance::<Empty>(
        pos_creator_setup.pos_creator_wrapper.address_ref(),
        DUAL_YIELD_TOKEN_ID,
        1,
        &rust_biguint!(0),
        None,
    );
    b_mock.borrow().check_nft_balance::<Empty>(
        pos_creator_setup.pos_creator_wrapper.address_ref(),
        STAKING_FARM_TOKEN_ID,
        2,
        &rust_biguint!(0),
        None,
    );
}

#[test]
fn enter_metastaking_with_merge_through_pos_creator_test() {
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
    let user_third_token_balance = 300_000_000u64;
    b_mock.borrow_mut().set_esdt_balance(
        &user_addr,
        TOKEN_IDS[2],
        &rust_biguint!(user_third_token_balance),
    );

    // user enter (A, B) metastaking farm with token C
    let second_pair_addr = pos_creator_setup.pair_setups[1]
        .pair_wrapper
        .address_ref()
        .clone();
    let ms_addr = pos_creator_setup.ms_wrapper.address_ref().clone();
    let expected_dual_yield_tokens = 22_727_271u64;
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
                    managed_token_id!(TOKEN_IDS[0]), // Want token B
                    BigUint::from(1u64),
                )
                    .into();
                swap_operations.push(swap_operation);
                let output_payments = sc.create_metastaking_pos_from_single_token(
                    managed_address!(&ms_addr),
                    1u32.into(),
                    1u32.into(),
                    swap_operations,
                );

                assert_eq!(
                    output_payments.get(1).token_identifier,
                    managed_token_id!(DUAL_YIELD_TOKEN_ID)
                );
                assert_eq!(
                    output_payments.get(1).amount,
                    managed_biguint!(expected_dual_yield_tokens)
                );
            },
        )
        .assert_ok();

    // Enter metastaking again, with the previous dual yield tokens, as additional payments
    // Use the same input LP token amount as the one obtained in the first operation
    let exact_input_amount = 23_255_813u64;
    b_mock.borrow_mut().set_esdt_balance(
        &user_addr,
        LP_TOKEN_IDS[0],
        &rust_biguint!(exact_input_amount),
    );
    let payments = vec![
        TxTokenTransfer {
            token_identifier: LP_TOKEN_IDS[0].to_vec(),
            nonce: 0,
            value: rust_biguint!(exact_input_amount),
        },
        TxTokenTransfer {
            token_identifier: DUAL_YIELD_TOKEN_ID.to_vec(),
            nonce: 1,
            value: rust_biguint!(expected_dual_yield_tokens),
        },
    ];

    b_mock
        .borrow_mut()
        .execute_esdt_multi_transfer(
            &user_addr,
            &pos_creator_setup.pos_creator_wrapper,
            &payments,
            |sc| {
                let output_payments = sc.create_metastaking_pos_from_single_token(
                    managed_address!(&ms_addr),
                    1u32.into(),
                    1u32.into(),
                    MultiValueEncoded::new(),
                );

                // There should be only 1 output payment, as we provided directly LP tokens
                // The amount should be equal with double the first obtained amount
                assert_eq!(output_payments.len(), 1);
                assert_eq!(
                    output_payments.get(0).token_identifier,
                    managed_token_id!(DUAL_YIELD_TOKEN_ID)
                );
                assert_eq!(
                    output_payments.get(0).amount,
                    managed_biguint!(expected_dual_yield_tokens * 2)
                );
            },
        )
        .assert_ok();
}

#[test]
fn create_metastaking_pos_from_two_tokens_test() {
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

    let expected_dual_yield_tokens = 166_666_666u64;
    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, TOKEN_IDS[0], &rust_biguint!(0));
    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, TOKEN_IDS[1], &rust_biguint!(0));
    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, TOKEN_IDS[2], &rust_biguint!(0));
    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, LP_TOKEN_IDS[0], &rust_biguint!(0));
    b_mock.borrow().check_nft_balance::<Empty>(
        &user_addr,
        FARM_TOKEN_ID[0],
        1,
        &rust_biguint!(0),
        None,
    );
    b_mock.borrow().check_nft_balance::<Empty>(
        &user_addr,
        DUAL_YIELD_TOKEN_ID,
        1,
        &rust_biguint!(expected_dual_yield_tokens),
        None,
    );
}

#[test]
fn enter_farm_staking_through_pos_creator_test() {
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
    let payment_token_balance = 1_000u64;
    let expected_output_amount = 166u64;
    b_mock.borrow_mut().set_esdt_balance(
        &user_addr,
        TOKEN_IDS[2],
        &rust_biguint!(payment_token_balance),
    );
    let second_pair_addr = pos_creator_setup.pair_setups[1]
        .pair_wrapper
        .address_ref()
        .clone();

    // user enters farm staking (A) with token C
    let farm_staking_address = pos_creator_setup.fs_wrapper.address_ref();
    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &user_addr,
            &pos_creator_setup.pos_creator_wrapper,
            TOKEN_IDS[2],
            0,
            &rust_biguint!(payment_token_balance),
            |sc| {
                // swap_operation -> pair_address, function, token_wanted, amount
                let mut swap_operations = MultiValueEncoded::new();
                let swap_operation: SwapOperationType<DebugApi> = (
                    managed_address!(&second_pair_addr),
                    ManagedBuffer::from(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME),
                    managed_token_id!(TOKEN_IDS[0]), // Want token B
                    BigUint::from(1u64),
                )
                    .into();
                swap_operations.push(swap_operation);
                let output_payments = sc.create_farm_staking_pos_from_single_token(
                    managed_address!(farm_staking_address),
                    managed_biguint!(expected_output_amount),
                    swap_operations,
                );

                assert_eq!(
                    output_payments.get(0).token_identifier,
                    managed_token_id!(STAKING_FARM_TOKEN_ID)
                );
                assert_eq!(output_payments.get(0).token_nonce, 1);
                assert_eq!(output_payments.get(0).amount, expected_output_amount);
            },
        )
        .assert_ok();

    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, TOKEN_IDS[0], &rust_biguint!(0));
    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, TOKEN_IDS[2], &rust_biguint!(0));
    b_mock
        .borrow()
        .check_nft_balance::<StakingFarmTokenAttributes<DebugApi>>(
            &user_addr,
            STAKING_FARM_TOKEN_ID,
            1,
            &rust_biguint!(expected_output_amount),
            None,
        );

    b_mock.borrow().check_esdt_balance(
        pos_creator_setup.pos_creator_wrapper.address_ref(),
        TOKEN_IDS[0],
        &rust_biguint!(0),
    );
    b_mock.borrow().check_esdt_balance(
        pos_creator_setup.pos_creator_wrapper.address_ref(),
        TOKEN_IDS[2],
        &rust_biguint!(0),
    );
    b_mock
        .borrow()
        .check_nft_balance::<StakingFarmTokenAttributes<DebugApi>>(
            pos_creator_setup.pos_creator_wrapper.address_ref(),
            STAKING_FARM_TOKEN_ID,
            1,
            &rust_biguint!(0),
            None,
        );
}

#[test]
fn create_pos_with_farm_boosted_rewards_test() {
    let mut pos_creator_setup = PosCreatorSetup::new(
        farm_with_locked_rewards::contract_obj,
        energy_factory::contract_obj,
        pair::contract_obj,
        router::contract_obj,
        farm_staking::contract_obj,
        farm_staking_proxy::contract_obj,
        auto_pos_creator::contract_obj,
    );

    let b_mock = pos_creator_setup.farm_setup.b_mock.clone();

    let user_addr = pos_creator_setup.farm_setup.first_user.clone();
    let user2_addr = pos_creator_setup.farm_setup.second_user.clone();
    let user_first_token_balance = 100_000_000u64;
    let user_second_token_balance = 200_000_000u64;
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
    b_mock
        .borrow_mut()
        .set_esdt_balance(&user2_addr, TOKEN_IDS[0], &rust_biguint!(1));

    pos_creator_setup.pair_setups[0].add_liquidity(
        &user_addr,
        user_first_token_balance,
        user_second_token_balance,
    );

    let lp_balance = 100_000_000u32;
    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, LP_TOKEN_IDS[0], &rust_biguint!(lp_balance));

    // Energy setup

    b_mock.borrow_mut().set_block_epoch(2);

    // first user enter farm
    let farm_address = pos_creator_setup.farm_setup.farm_wrappers[0]
        .address_ref()
        .clone();
    let ms_address = pos_creator_setup.ms_wrapper.address_ref().clone();
    pos_creator_setup
        .farm_setup
        .set_user_energy(&user_addr, 1_000, 2, 1);
    // User creates farm pos with farming token directly
    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &user_addr,
            &pos_creator_setup.pos_creator_wrapper,
            LP_TOKEN_IDS[0],
            0,
            &rust_biguint!(lp_balance / 2),
            |sc| {
                sc.create_farm_pos_from_single_token(
                    managed_address!(&farm_address),
                    1u32.into(),
                    1u32.into(),
                    MultiValueEncoded::new(),
                );
            },
        )
        .assert_ok();

    // second user enter farm
    b_mock.borrow_mut().set_block_nonce(10);

    // random tx on end of week 1, to cummulate rewards
    b_mock.borrow_mut().set_block_epoch(6);
    pos_creator_setup
        .farm_setup
        .set_user_energy(&user_addr, 1_000, 6, 1);
    pos_creator_setup
        .farm_setup
        .set_user_energy(&user2_addr, 1, 6, 1);

    pos_creator_setup.farm_setup.enter_farm(0, &user2_addr, 1);
    pos_creator_setup.farm_setup.exit_farm(0, &user2_addr, 2, 1);

    // advance 1 week
    b_mock.borrow_mut().set_block_epoch(10);
    pos_creator_setup
        .farm_setup
        .set_user_energy(&user_addr, 1_000, 10, 1);

    b_mock.borrow().check_nft_balance::<Empty>(
        &user_addr,
        LOCKED_REWARD_TOKEN_ID,
        1,
        &rust_biguint!(0),
        None,
    );

    // On new enter (including Metastaking), user should receive boosted rewards from farm
    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &user_addr,
            &pos_creator_setup.pos_creator_wrapper,
            LP_TOKEN_IDS[0],
            0,
            &rust_biguint!(lp_balance / 2),
            |sc| {
                sc.create_metastaking_pos_from_single_token(
                    managed_address!(&ms_address),
                    1u32.into(),
                    1u32.into(),
                    MultiValueEncoded::new(),
                );
            },
        )
        .assert_ok();

    // Check dual yield token balance
    b_mock.borrow().check_nft_balance::<Empty>(
        &user_addr,
        DUAL_YIELD_TOKEN_ID,
        1,
        &rust_biguint!(45_454_545u64),
        None,
    );

    // Check boosted rewards
    b_mock.borrow().check_nft_balance::<Empty>(
        &user_addr,
        LOCKED_REWARD_TOKEN_ID,
        1,
        &rust_biguint!(2_500u64),
        None,
    );
}

#[test]
fn create_farm_from_lp_test() {
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

    let first_pair_addr = pos_creator_setup.pair_setups[0]
        .pair_wrapper
        .address_ref()
        .clone();
    let second_pair_addr = pos_creator_setup.pair_setups[1]
        .pair_wrapper
        .address_ref()
        .clone();

    // user enter (A, B) LP with token C
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

                let _ = sc.create_lp_pos_from_single_token(
                    managed_address!(&first_pair_addr),
                    1u32.into(),
                    1u32.into(),
                    swap_operations,
                );
            },
        )
        .assert_ok();

    let expected_lp_tokens = 43_478_260u64;
    b_mock.borrow().check_esdt_balance(
        &user_addr,
        LP_TOKEN_IDS[0],
        &rust_biguint!(expected_lp_tokens),
    );

    let farm_addr = pos_creator_setup.farm_setup.farm_wrappers[0]
        .address_ref()
        .clone();

    // user enter farm from LP token
    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &user_addr,
            &pos_creator_setup.pos_creator_wrapper,
            LP_TOKEN_IDS[0],
            0,
            &rust_biguint!(expected_lp_tokens),
            |sc| {
                let _ = sc.create_farm_pos_from_single_token(
                    managed_address!(&farm_addr),
                    1u32.into(),
                    1u32.into(),
                    MultiValueEncoded::new(),
                );
            },
        )
        .assert_ok();

    b_mock.borrow().check_nft_balance::<Empty>(
        &user_addr,
        FARM_TOKEN_ID[0],
        1,
        &rust_biguint!(expected_lp_tokens),
        None,
    );
}

#[test]
fn enter_lp_through_pos_creator_long_swap_path_test() {
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
                let mut swap_operations = MultiValueEncoded::new();
                let swap_operation: SwapOperationType<DebugApi> = (
                    managed_address!(&first_pair_addr),
                    ManagedBuffer::from(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME),
                    managed_token_id!(TOKEN_IDS[1]), // Want token B
                    BigUint::from(1u64),
                )
                    .into();
                swap_operations.push(swap_operation);

                let second_swap_operation: SwapOperationType<DebugApi> = (
                    managed_address!(&third_pair_addr),
                    ManagedBuffer::from(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME),
                    managed_token_id!(TOKEN_IDS[2]), // Want token C
                    BigUint::from(1u64),
                )
                    .into();
                swap_operations.push(second_swap_operation);

                let third_swap_operation: SwapOperationType<DebugApi> = (
                    managed_address!(&third_pair_addr),
                    ManagedBuffer::from(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME),
                    managed_token_id!(TOKEN_IDS[1]), // Want token B
                    BigUint::from(1u64),
                )
                    .into();
                swap_operations.push(third_swap_operation);

                let _ = sc.create_lp_pos_from_single_token(
                    managed_address!(&third_pair_addr),
                    1u32.into(),
                    1u32.into(),
                    swap_operations,
                );
            },
        )
        .assert_ok();

    // User adds liquidity in pool B-C, using token A
    // Route: All tokens A are swapped to token B
    // All tokens B are swapped to C
    // All tokens C are swapped to B
    // Half of the swap output is then swapped to token C
    // Add liquidity using the resulted tokens B and C
    // Same amount, as fees are set to 0
    let expected_lp_token = 142_857_142u64;
    b_mock.borrow().check_esdt_balance(
        &user_addr,
        LP_TOKEN_IDS[2],
        &rust_biguint!(expected_lp_token),
    );
}

#[test]
fn try_create_lp_pos_from_same_lp_token_test() {
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
                let mut swap_operations = MultiValueEncoded::new();
                let swap_operation: SwapOperationType<DebugApi> = (
                    managed_address!(&first_pair_addr),
                    ManagedBuffer::from(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME),
                    managed_token_id!(TOKEN_IDS[1]), // Want token B
                    BigUint::from(1u64),
                )
                    .into();
                swap_operations.push(swap_operation);

                let second_swap_operation: SwapOperationType<DebugApi> = (
                    managed_address!(&third_pair_addr),
                    ManagedBuffer::from(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME),
                    managed_token_id!(TOKEN_IDS[2]), // Want token C
                    BigUint::from(1u64),
                )
                    .into();
                swap_operations.push(second_swap_operation);

                let third_swap_operation: SwapOperationType<DebugApi> = (
                    managed_address!(&third_pair_addr),
                    ManagedBuffer::from(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME),
                    managed_token_id!(TOKEN_IDS[1]), // Want token B
                    BigUint::from(1u64),
                )
                    .into();
                swap_operations.push(third_swap_operation);

                let _ = sc.create_lp_pos_from_single_token(
                    managed_address!(&third_pair_addr),
                    1u32.into(),
                    1u32.into(),
                    swap_operations,
                );
            },
        )
        .assert_ok();

    // User adds liquidity in pool B-C, using token A
    // Route: All tokens A are swapped to token B
    // All tokens B are swapped to C
    // All tokens C are swapped to B
    // Half of the swap output is then swapped to token C
    // Add liquidity using the resulted tokens B and C
    // Same amount, as fees are set to 0
    let expected_lp_token = 142_857_142u64;
    b_mock.borrow().check_esdt_balance(
        &user_addr,
        LP_TOKEN_IDS[2],
        &rust_biguint!(expected_lp_token),
    );

    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &user_addr,
            &pos_creator_setup.pos_creator_wrapper,
            LP_TOKEN_IDS[2],
            0,
            &rust_biguint!(expected_lp_token),
            |sc| {
                let _ = sc.create_lp_pos_from_single_token(
                    managed_address!(&third_pair_addr),
                    1u32.into(),
                    1u32.into(),
                    MultiValueEncoded::new(),
                );
            },
        )
        .assert_ok();

    b_mock.borrow().check_esdt_balance(
        &user_addr,
        LP_TOKEN_IDS[2],
        &rust_biguint!(expected_lp_token),
    );
}

#[test]
fn try_create_lp_pos_from_different_lp_token_test() {
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
                let mut swap_operations = MultiValueEncoded::new();
                let swap_operation: SwapOperationType<DebugApi> = (
                    managed_address!(&first_pair_addr),
                    ManagedBuffer::from(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME),
                    managed_token_id!(TOKEN_IDS[1]), // Want token B
                    BigUint::from(1u64),
                )
                    .into();
                swap_operations.push(swap_operation);

                let second_swap_operation: SwapOperationType<DebugApi> = (
                    managed_address!(&third_pair_addr),
                    ManagedBuffer::from(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME),
                    managed_token_id!(TOKEN_IDS[2]), // Want token C
                    BigUint::from(1u64),
                )
                    .into();
                swap_operations.push(second_swap_operation);

                let third_swap_operation: SwapOperationType<DebugApi> = (
                    managed_address!(&third_pair_addr),
                    ManagedBuffer::from(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME),
                    managed_token_id!(TOKEN_IDS[1]), // Want token B
                    BigUint::from(1u64),
                )
                    .into();
                swap_operations.push(third_swap_operation);

                let _ = sc.create_lp_pos_from_single_token(
                    managed_address!(&third_pair_addr),
                    1u32.into(),
                    1u32.into(),
                    swap_operations,
                );
            },
        )
        .assert_ok();

    // User adds liquidity in pool B-C, using token A
    // Route: All tokens A are swapped to token B
    // All tokens B are swapped to C
    // All tokens C are swapped to B
    // Half of the swap output is then swapped to token C
    // Add liquidity using the resulted tokens B and C
    // Same amount, as fees are set to 0
    let expected_lp_token = 142_857_142u64;
    b_mock.borrow().check_esdt_balance(
        &user_addr,
        LP_TOKEN_IDS[2],
        &rust_biguint!(expected_lp_token),
    );

    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &user_addr,
            &pos_creator_setup.pos_creator_wrapper,
            LP_TOKEN_IDS[2],
            0,
            &rust_biguint!(expected_lp_token),
            |sc| {
                let _ = sc.create_lp_pos_from_single_token(
                    managed_address!(&first_pair_addr),
                    1u32.into(),
                    1u32.into(),
                    MultiValueEncoded::new(),
                );
            },
        )
        .assert_user_error("The output token identifier is not part of the LP");
}

#[test]
fn user_exit_metastaking_with_penalty_test() {
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

    b_mock
        .borrow_mut()
        .execute_tx(
            &pos_creator_setup.farm_setup.owner,
            &pos_creator_setup.farm_setup.farm_wrappers[0],
            &rust_biguint!(0),
            |sc| {
                sc.penalty_percent().set(1_000);
                sc.minimum_farming_epochs().set(10);
            },
        )
        .assert_ok();

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

    let expected_dual_yield_tokens = 166_666_666u64;
    b_mock.borrow().check_nft_balance::<Empty>(
        &user_addr,
        DUAL_YIELD_TOKEN_ID,
        1,
        &rust_biguint!(expected_dual_yield_tokens),
        None,
    );

    // set address to 0 so tokens are burned directly
    b_mock
        .borrow_mut()
        .execute_tx(
            &pos_creator_setup.farm_setup.owner,
            &pos_creator_setup.farm_setup.farm_wrappers[0],
            &rust_biguint!(0),
            |sc| {
                sc.pair_contract_address().set(ManagedAddress::zero());
            },
        )
        .assert_ok();

    // user exit metastaking pos
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
                    managed_address!(&ms_addr),
                    1u32.into(),
                    1u32.into(),
                );
            },
        )
        .assert_ok();

    // 10% penalty - from 400M and 200M
    let expected_second_token_amount_from_lp = 360_000_000u64;
    let expected_staking_farm_token_amount = 180_000_000u64;

    b_mock.borrow().check_esdt_balance(
        &user_addr,
        TOKEN_IDS[1],
        &rust_biguint!(expected_second_token_amount_from_lp),
    );
    b_mock.borrow().check_nft_balance(
        &user_addr,
        STAKING_FARM_TOKEN_ID,
        2,
        &rust_biguint!(expected_staking_farm_token_amount),
        Some(&UnbondSftAttributes { unlock_epoch: 5 }),
    );
}

#[test]
fn user_exit_metastaking_without_penalty_test() {
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

    let expected_dual_yield_tokens = 166_666_666u64;
    b_mock.borrow().check_nft_balance::<Empty>(
        &user_addr,
        DUAL_YIELD_TOKEN_ID,
        1,
        &rust_biguint!(expected_dual_yield_tokens),
        None,
    );

    // user exit metastaking pos
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
                    managed_address!(&ms_addr),
                    1u32.into(),
                    1u32.into(),
                );
            },
        )
        .assert_ok();

    // No penalty
    let expected_second_token_amount_from_lp = user_second_token_balance;
    let expected_staking_farm_token_amount = user_first_token_balance;

    b_mock.borrow().check_esdt_balance(
        &user_addr,
        TOKEN_IDS[1],
        &rust_biguint!(expected_second_token_amount_from_lp),
    );
    b_mock.borrow().check_nft_balance(
        &user_addr,
        STAKING_FARM_TOKEN_ID,
        2,
        &rust_biguint!(expected_staking_farm_token_amount),
        Some(&UnbondSftAttributes { unlock_epoch: 5 }),
    );
}
