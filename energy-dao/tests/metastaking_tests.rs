#![allow(deprecated)]

mod contract_interactions;
mod contract_setup;

use contract_setup::*;
use energy_dao::common::structs::{
    UnstakeMetastakingAttributes, WrappedFarmTokenAttributes, WrappedMetastakingTokenAttributes,
};
use multiversx_sc_scenario::{rust_biguint, DebugApi};

#[test]
fn energy_dao_metastaking_test() {
    let mut energy_dao_setup = EnergyDAOContractSetup::new(
        energy_dao::contract_obj,
        energy_factory::contract_obj,
        fees_collector::contract_obj,
        locked_token_wrapper::contract_obj,
        pair::contract_obj,
        farm_with_locked_rewards::contract_obj,
        farm_staking::contract_obj,
        farm_staking_proxy::contract_obj,
    );

    // necessary to initialize the liquidity pool
    energy_dao_setup.b_mock.set_block_nonce(1);
    energy_dao_setup.b_mock.set_block_round(1);

    let farm_address = energy_dao_setup.farm_wrapper.address_ref().clone();
    let farm_staking_proxy_address = energy_dao_setup
        .farm_staking_proxy_wrapper
        .address_ref()
        .clone();
    energy_dao_setup.add_farm(&farm_address);
    energy_dao_setup.add_metastaking_address(&farm_staking_proxy_address);
    energy_dao_setup.lock_energy_tokens(USER_BALANCE, LOCK_OPTIONS[2]);
    let user1_base_token_amount = 100_000_000_000u64;
    let user1_other_token_amount = user1_base_token_amount / 100;
    let user2_base_token_amount = 50_000_000_000u64;
    let user2_other_token_amount = user2_base_token_amount / 100;
    let user1 =
        energy_dao_setup.setup_new_user(BASE_FARM_STAKING_TOKEN_ID, user1_base_token_amount);
    let user2 =
        energy_dao_setup.setup_new_user(BASE_FARM_STAKING_TOKEN_ID, user2_base_token_amount);
    energy_dao_setup.b_mock.set_esdt_balance(
        &user1,
        OTHER_FARM_STAKING_TOKEN_ID,
        &rust_biguint!(user1_base_token_amount / 100),
    );
    energy_dao_setup.b_mock.set_esdt_balance(
        &user2,
        OTHER_FARM_STAKING_TOKEN_ID,
        &rust_biguint!(user2_base_token_amount / 100),
    );

    // add initial liquidity
    let liquidity_user = energy_dao_setup
        .b_mock
        .create_user_account(&rust_biguint!(0));
    energy_dao_setup.b_mock.set_esdt_balance(
        &liquidity_user,
        BASE_FARM_STAKING_TOKEN_ID,
        &rust_biguint!(user1_base_token_amount + user2_base_token_amount),
    );
    energy_dao_setup.b_mock.set_esdt_balance(
        &liquidity_user,
        OTHER_FARM_STAKING_TOKEN_ID,
        &rust_biguint!(user1_other_token_amount + user2_other_token_amount),
    );
    let liq_user_lp_amount = energy_dao_setup.call_pair_add_liquidity(
        &liquidity_user,
        BASE_FARM_STAKING_TOKEN_ID,
        user1_base_token_amount + user2_base_token_amount,
        OTHER_FARM_STAKING_TOKEN_ID,
        user1_other_token_amount + user2_other_token_amount,
    );
    energy_dao_setup.call_pair_remove_liquidity(
        &liquidity_user,
        FARMING_TOKEN_ID,
        liq_user_lp_amount - 1_000u64, // 1000 - min liquidity
    );

    // Users enter liquidity on multiple blocks for LP safe price computation
    energy_dao_setup.b_mock.set_block_nonce(5);
    energy_dao_setup.b_mock.set_block_round(5);

    let user1_lp_amount = energy_dao_setup.call_pair_add_liquidity(
        &user1,
        BASE_FARM_STAKING_TOKEN_ID,
        user1_base_token_amount,
        OTHER_FARM_STAKING_TOKEN_ID,
        user1_other_token_amount,
    );

    energy_dao_setup.b_mock.set_block_nonce(10);
    energy_dao_setup.b_mock.set_block_round(10);

    let user2_lp_amount = energy_dao_setup.call_pair_add_liquidity(
        &user2,
        BASE_FARM_STAKING_TOKEN_ID,
        user2_base_token_amount,
        OTHER_FARM_STAKING_TOKEN_ID,
        user2_other_token_amount,
    );

    // The user enters metastaking with the LP position directly
    let user1_metastaking_amount = energy_dao_setup.enter_energy_dao_metastaking_endpoint(
        &farm_staking_proxy_address,
        &user1,
        FARMING_TOKEN_ID,
        user1_lp_amount,
    );
    let user2_metastaking_amount = energy_dao_setup.enter_energy_dao_metastaking_endpoint(
        &farm_staking_proxy_address,
        &user2,
        FARMING_TOKEN_ID,
        user2_lp_amount,
    );

    energy_dao_setup
        .b_mock
        .check_nft_balance::<WrappedMetastakingTokenAttributes<DebugApi>>(
            &user1,
            WRAPPED_METASTAKING_TOKEN_ID,
            1,
            &num_bigint::BigUint::from(user1_metastaking_amount),
            None,
        );
    energy_dao_setup
        .b_mock
        .check_nft_balance::<WrappedMetastakingTokenAttributes<DebugApi>>(
            &user2,
            WRAPPED_METASTAKING_TOKEN_ID,
            2,
            &num_bigint::BigUint::from(user2_metastaking_amount),
            None,
        );

    energy_dao_setup.b_mock.set_block_epoch(10u64);
    energy_dao_setup.b_mock.set_block_nonce(110u64);
    energy_dao_setup.b_mock.set_block_round(110u64);

    energy_dao_setup.claim_user_metastaking_rewards(
        &user1,
        WRAPPED_METASTAKING_TOKEN_ID,
        1,
        user1_metastaking_amount,
    );
    energy_dao_setup.claim_user_metastaking_rewards(
        &user2,
        WRAPPED_METASTAKING_TOKEN_ID,
        2,
        user2_metastaking_amount,
    );

    // Check locked rewards
    let user1_locked_rewards = 250_000u64;
    let user2_locked_rewards = user1_locked_rewards / 2;
    energy_dao_setup
        .b_mock
        .check_nft_balance::<WrappedFarmTokenAttributes<DebugApi>>(
            &user1,
            WRAPPED_LOCKED_TOKEN_ID,
            1,
            &num_bigint::BigUint::from(user1_locked_rewards),
            None,
        );

    energy_dao_setup
        .b_mock
        .check_nft_balance::<WrappedFarmTokenAttributes<DebugApi>>(
            &user2,
            WRAPPED_LOCKED_TOKEN_ID,
            1,
            &num_bigint::BigUint::from(user2_locked_rewards),
            None,
        );

    // Check staking rewards
    let user1_staking_rewards = 148_000u64;
    let user2_staking_rewards = user1_staking_rewards / 2;
    energy_dao_setup.b_mock.check_esdt_balance(
        &user1,
        BASE_FARM_STAKING_TOKEN_ID,
        &num_bigint::BigUint::from(user1_staking_rewards),
    );

    energy_dao_setup.b_mock.check_esdt_balance(
        &user2,
        BASE_FARM_STAKING_TOKEN_ID,
        &num_bigint::BigUint::from(user2_staking_rewards),
    );

    energy_dao_setup
        .b_mock
        .check_nft_balance::<WrappedMetastakingTokenAttributes<DebugApi>>(
            &user1,
            WRAPPED_METASTAKING_TOKEN_ID,
            3,
            &num_bigint::BigUint::from(user1_metastaking_amount),
            None,
        );
    energy_dao_setup
        .b_mock
        .check_nft_balance::<WrappedMetastakingTokenAttributes<DebugApi>>(
            &user2,
            WRAPPED_METASTAKING_TOKEN_ID,
            4,
            &num_bigint::BigUint::from(user2_metastaking_amount),
            None,
        );

    // user 1 exits the contract during the same block nonce
    energy_dao_setup.unstake_metastaking(
        &user1,
        WRAPPED_METASTAKING_TOKEN_ID,
        3,
        user1_metastaking_amount,
    );

    let user1_unstake_amount = 99_999_999_900u64;
    energy_dao_setup
        .b_mock
        .check_nft_balance::<UnstakeMetastakingAttributes<DebugApi>>(
            &user1,
            UNSTAKE_METASTAKING_TOKEN_ID,
            1,
            &num_bigint::BigUint::from(user1_unstake_amount),
            None,
        );

    energy_dao_setup.b_mock.set_block_epoch(20u64);

    energy_dao_setup.unbond_metastaking(
        &user1,
        UNSTAKE_METASTAKING_TOKEN_ID,
        1,
        user1_unstake_amount,
    );

    energy_dao_setup
        .b_mock
        .check_nft_balance::<WrappedFarmTokenAttributes<DebugApi>>(
            &user1,
            WRAPPED_LOCKED_TOKEN_ID,
            1,
            &num_bigint::BigUint::from(user1_locked_rewards),
            None,
        );

    let user1_base_token_fee = 2_999_999_997u64;
    energy_dao_setup.b_mock.check_esdt_balance(
        &user1,
        BASE_FARM_STAKING_TOKEN_ID,
        &num_bigint::BigUint::from(
            user1_staking_rewards + user1_unstake_amount - user1_base_token_fee,
        ),
    );

    let user1_other_token_balance = 970_000_000u64;
    energy_dao_setup.b_mock.check_esdt_balance(
        &user1,
        OTHER_FARM_STAKING_TOKEN_ID,
        &num_bigint::BigUint::from(user1_other_token_balance),
    );

    // user 2 exits the contract after some more blocks
    energy_dao_setup.b_mock.set_block_nonce(210u64);
    energy_dao_setup.b_mock.set_block_round(210u64);

    energy_dao_setup.unstake_metastaking(
        &user2,
        WRAPPED_METASTAKING_TOKEN_ID,
        4,
        user2_metastaking_amount,
    );

    let user2_unstake_amount = 50_000_000_000u64;
    energy_dao_setup
        .b_mock
        .check_nft_balance::<UnstakeMetastakingAttributes<DebugApi>>(
            &user2,
            UNSTAKE_METASTAKING_TOKEN_ID,
            2,
            &num_bigint::BigUint::from(user2_unstake_amount),
            None,
        );

    energy_dao_setup.b_mock.set_block_epoch(30u64);

    energy_dao_setup.unbond_metastaking(
        &user2,
        UNSTAKE_METASTAKING_TOKEN_ID,
        2,
        user2_unstake_amount,
    );

    let user2_new_locked_rewards = 499_500u64;

    energy_dao_setup
        .b_mock
        .check_nft_balance::<WrappedFarmTokenAttributes<DebugApi>>(
            &user2,
            WRAPPED_LOCKED_TOKEN_ID,
            1,
            &num_bigint::BigUint::from(user2_new_locked_rewards),
            None,
        );

    let user2_new_staking_rewards = 130_000u64;
    let user2_base_token_fee = 1_499_940_500u64;

    energy_dao_setup.b_mock.check_esdt_balance(
        &user2,
        BASE_FARM_STAKING_TOKEN_ID,
        &num_bigint::BigUint::from(
            user2_staking_rewards + user2_new_staking_rewards + user2_unstake_amount
                - user2_base_token_fee,
        ),
    );

    let user2_other_token_balance = 485_000_000u64;
    energy_dao_setup.b_mock.check_esdt_balance(
        &user2,
        OTHER_FARM_STAKING_TOKEN_ID,
        &num_bigint::BigUint::from(user2_other_token_balance),
    );
}
