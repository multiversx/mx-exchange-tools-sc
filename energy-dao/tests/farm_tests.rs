mod contract_interactions;
mod contract_setup;

use common_structs::FarmTokenAttributes;
use contract_setup::*;
use energy_dao::{
    common::structs::{UnstakeFarmAttributes, WrappedFarmTokenAttributes},
    external_sc_interactions::energy_dao_config::MAX_PERCENTAGE,
};
use multiversx_sc_scenario::DebugApi;

#[test]
fn init_test() {
    let _ = EnergyDAOContractSetup::new(
        energy_dao::contract_obj,
        energy_factory::contract_obj,
        fees_collector::contract_obj,
        locked_token_wrapper::contract_obj,
        pair::contract_obj,
        farm_with_locked_rewards::contract_obj,
        farm_staking::contract_obj,
        farm_staking_proxy::contract_obj,
    );
}

#[test]
fn lock_tokens_test() {
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

    energy_dao_setup.lock_energy_tokens(USER_BALANCE, LOCK_OPTIONS[2])
}

#[test]
fn energy_dao_enter_exit_with_penalty_test() {
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
    let farm_address = energy_dao_setup.farm_wrapper.address_ref().clone();
    energy_dao_setup.add_farm(&farm_address);

    let user_farm_amount = 1_000u64;
    let user1 = energy_dao_setup.setup_new_user(FARMING_TOKEN_ID, user_farm_amount);
    energy_dao_setup.enter_energy_dao_farm_endpoint(
        &farm_address,
        &user1,
        FARMING_TOKEN_ID,
        user_farm_amount,
    );

    energy_dao_setup
        .b_mock
        .check_nft_balance::<FarmTokenAttributes<DebugApi>>(
            energy_dao_setup.energy_dao_wrapper.address_ref(),
            FARM_TOKEN_ID,
            1,
            &num_bigint::BigUint::from(user_farm_amount),
            None,
        );

    energy_dao_setup
        .b_mock
        .check_nft_balance::<WrappedFarmTokenAttributes<DebugApi>>(
            &user1,
            WRAPPED_FARM_TOKEN_ID,
            1,
            &num_bigint::BigUint::from(user_farm_amount),
            None,
        );

    energy_dao_setup.unstake_farm(&user1, WRAPPED_FARM_TOKEN_ID, 1, user_farm_amount);

    // check if tokens were burned
    energy_dao_setup
        .b_mock
        .check_nft_balance::<WrappedFarmTokenAttributes<DebugApi>>(
            energy_dao_setup.energy_dao_wrapper.address_ref(),
            WRAPPED_FARM_TOKEN_ID,
            1,
            &num_bigint::BigUint::from(0u64),
            None,
        );

    energy_dao_setup
        .b_mock
        .check_nft_balance::<UnstakeFarmAttributes<DebugApi>>(
            &user1,
            UNSTAKE_TOKEN_ID,
            1,
            &num_bigint::BigUint::from(user_farm_amount),
            None,
        );

    energy_dao_setup.b_mock.set_block_epoch(10);
    energy_dao_setup.unbond_farm(&user1, UNSTAKE_TOKEN_ID, 1, user_farm_amount);

    // check if tokens were burned
    energy_dao_setup
        .b_mock
        .check_nft_balance::<WrappedFarmTokenAttributes<DebugApi>>(
            energy_dao_setup.energy_dao_wrapper.address_ref(),
            UNSTAKE_TOKEN_ID,
            1,
            &num_bigint::BigUint::from(0u64),
            None,
        );

    energy_dao_setup.b_mock.check_esdt_balance(
        &user1,
        FARMING_TOKEN_ID,
        &num_bigint::BigUint::from(
            user_farm_amount - (user_farm_amount * PENALTY_PERCENTAGE / MAX_PERCENTAGE),
        ),
    );
}

#[test]
fn energy_dao_multiple_users_with_claim_test() {
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
    let farm_address = energy_dao_setup.farm_wrapper.address_ref().clone();
    energy_dao_setup.add_farm(&farm_address);
    energy_dao_setup.lock_energy_tokens(USER_BALANCE, LOCK_OPTIONS[2]);

    let user1_farm_amount = 10_000u64;
    let user2_farm_amount = 5_000u64;
    let user1 = energy_dao_setup.setup_new_user(FARMING_TOKEN_ID, user1_farm_amount);
    let user2 = energy_dao_setup.setup_new_user(FARMING_TOKEN_ID, user2_farm_amount);
    energy_dao_setup.enter_energy_dao_farm_endpoint(
        &farm_address,
        &user1,
        FARMING_TOKEN_ID,
        user1_farm_amount,
    );
    energy_dao_setup.enter_energy_dao_farm_endpoint(
        &farm_address,
        &user2,
        FARMING_TOKEN_ID,
        user2_farm_amount,
    );

    energy_dao_setup
        .b_mock
        .check_nft_balance::<FarmTokenAttributes<DebugApi>>(
            energy_dao_setup.energy_dao_wrapper.address_ref(),
            FARM_TOKEN_ID,
            3,
            &num_bigint::BigUint::from(user1_farm_amount + user2_farm_amount),
            None,
        );

    energy_dao_setup
        .b_mock
        .check_nft_balance::<WrappedFarmTokenAttributes<DebugApi>>(
            &user1,
            WRAPPED_FARM_TOKEN_ID,
            1,
            &num_bigint::BigUint::from(user1_farm_amount),
            None,
        );
    energy_dao_setup
        .b_mock
        .check_nft_balance::<WrappedFarmTokenAttributes<DebugApi>>(
            &user2,
            WRAPPED_FARM_TOKEN_ID,
            2,
            &num_bigint::BigUint::from(user2_farm_amount),
            None,
        );

    energy_dao_setup.b_mock.set_block_nonce(10u64);
    energy_dao_setup.claim_user_rewards(&user1, WRAPPED_FARM_TOKEN_ID, 1, user1_farm_amount);

    energy_dao_setup.b_mock.set_block_nonce(20u64);
    energy_dao_setup.claim_user_rewards(&user2, WRAPPED_FARM_TOKEN_ID, 2, user2_farm_amount);

    energy_dao_setup
        .b_mock
        .check_nft_balance::<WrappedFarmTokenAttributes<DebugApi>>(
            &user1,
            WRAPPED_LOCKED_TOKEN_ID,
            1,
            &num_bigint::BigUint::from(25_000u64),
            None,
        );

    energy_dao_setup
        .b_mock
        .check_nft_balance::<WrappedFarmTokenAttributes<DebugApi>>(
            &user2,
            WRAPPED_LOCKED_TOKEN_ID,
            1,
            &num_bigint::BigUint::from(25_000u64),
            None,
        );

    // token rps = 2_500_000u64
    energy_dao_setup
        .b_mock
        .check_nft_balance::<WrappedFarmTokenAttributes<DebugApi>>(
            &user1,
            WRAPPED_FARM_TOKEN_ID,
            3,
            &num_bigint::BigUint::from(user1_farm_amount),
            None,
        );

    // token rps = 5_000_000u64
    energy_dao_setup
        .b_mock
        .check_nft_balance::<WrappedFarmTokenAttributes<DebugApi>>(
            &user2,
            WRAPPED_FARM_TOKEN_ID,
            4,
            &num_bigint::BigUint::from(user2_farm_amount),
            None,
        );
}
