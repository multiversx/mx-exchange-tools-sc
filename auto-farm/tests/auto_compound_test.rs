use auto_farm::{
    common::{
        common_storage::MAX_PERCENTAGE,
        rewards_wrapper::{MergedRewardsWrapper, RewardsWrapper},
        unique_payments::UniquePayments,
    },
    external_sc_interactions::{
        fees_collector_actions::FeesCollectorActionsModule,
        multi_contract_interactions::MultiContractInteractionsModule,
    },
    fees::FeesModule,
    registration::RegistrationModule,
    user_tokens::{
        user_farm_tokens::{EndpointWrappers, UserFarmTokensModule},
        user_rewards::UserRewardsModule,
    },
    whitelists::farms_whitelist::FarmsWhitelistModule,
    AutoFarm,
};
use elrond_wasm::{
    elrond_codec::{multi_types::OptionalValue, Empty},
    types::{EsdtTokenPayment, ManagedVec, MultiValueEncoded},
};
use elrond_wasm_debug::{
    managed_address, managed_biguint, managed_token_id, rust_biguint, DebugApi,
};
use energy_factory::locked_token_transfer::LockedTokenTransferModule;
use farm_staking::stake_farm::StakeFarmModule;
use farm_staking_setup::{setup_farm_staking, STAKING_FARM_TOKEN_ID, USER_TOTAL_RIDE_TOKENS};
use farm_with_locked_rewards_setup::FarmSetup;
use fees_collector_setup::{
    setup_fees_collector, FIRST_TOKEN_ID, LOCKED_TOKEN_ID, SECOND_TOKEN_ID,
};
use sc_whitelist_module::SCWhitelistModule;

mod farm_staking_setup;
mod farm_with_locked_rewards_setup;
mod fees_collector_setup;

const FEE_PERCENTAGE: u64 = 1_000; // 10%

#[test]
fn farm_staking_setup_test() {
    let mut farm_setup = FarmSetup::new(
        farm_with_locked_rewards::contract_obj,
        energy_factory::contract_obj,
    );
    let _ = setup_farm_staking(
        &mut farm_setup.b_mock,
        farm_staking::contract_obj,
        FIRST_TOKEN_ID,
        FIRST_TOKEN_ID,
    );
}

#[test]
fn auto_compound_test() {
    let rust_zero = rust_biguint!(0);
    let mut farm_setup = FarmSetup::new(
        farm_with_locked_rewards::contract_obj,
        energy_factory::contract_obj,
    );
    let farm_staking_wrapper = setup_farm_staking(
        &mut farm_setup.b_mock,
        farm_staking::contract_obj,
        FIRST_TOKEN_ID,
        FIRST_TOKEN_ID,
    );

    farm_setup.b_mock.set_esdt_balance(
        &farm_setup.first_user,
        FIRST_TOKEN_ID,
        &rust_biguint!(USER_TOTAL_RIDE_TOKENS),
    );

    /////////////////////////////////////
    let owner = farm_setup.b_mock.create_user_account(&rust_zero);
    let proxy_address = farm_setup.b_mock.create_user_account(&rust_zero);
    let auto_farm_wrapper = farm_setup.b_mock.create_sc_account(
        &rust_zero,
        Some(&owner),
        auto_farm::contract_obj,
        "auto farm",
    );

    let energy_factory_addr = farm_setup.energy_factory_wrapper.address_ref().clone();
    let fc_wrapper = setup_fees_collector(
        &mut farm_setup.b_mock,
        fees_collector::contract_obj,
        &energy_factory_addr,
    );

    farm_setup
        .b_mock
        .execute_tx(&owner, &auto_farm_wrapper, &rust_zero, |sc| {
            sc.init(
                managed_address!(&proxy_address),
                FEE_PERCENTAGE,
                managed_address!(&energy_factory_addr),
                managed_address!(fc_wrapper.address_ref()), // unused here
                managed_address!(fc_wrapper.address_ref()),
            );

            sc.add_farms(
                ManagedVec::from_single_item(managed_address!(farm_staking_wrapper.address_ref()))
                    .into(),
            );
        })
        .assert_ok();

    // whitelist auto-farm SC in fees collector
    farm_setup
        .b_mock
        .execute_tx(&owner, &fc_wrapper, &rust_zero, |sc| {
            sc.sc_whitelist_addresses()
                .add(&managed_address!(auto_farm_wrapper.address_ref()))
        })
        .assert_ok();

    // whitelist auto-farm SC in farm-staking
    farm_setup
        .b_mock
        .execute_tx(&owner, &farm_staking_wrapper, &rust_zero, |sc| {
            sc.sc_whitelist_addresses()
                .add(&managed_address!(auto_farm_wrapper.address_ref()))
        })
        .assert_ok();

    // whitelist fees collector and auto-farm in energy factory
    farm_setup
        .b_mock
        .execute_tx(
            &farm_setup.owner,
            &farm_setup.energy_factory_wrapper,
            &rust_zero,
            |sc| {
                sc.add_to_token_transfer_whitelist(
                    ManagedVec::from_single_item(managed_address!(auto_farm_wrapper.address_ref()))
                        .into(),
                );

                sc.sc_whitelist_addresses()
                    .add(&managed_address!(fc_wrapper.address_ref()));
            },
        )
        .assert_ok();

    let first_user_addr = farm_setup.first_user.clone();
    let second_user_addr = farm_setup.second_user.clone();

    // replace with deposit farm staking token

    // user enter farm staking
    let farm_in_amount = rust_biguint!(100_000_000);
    farm_setup
        .b_mock
        .execute_esdt_transfer(
            &first_user_addr,
            &farm_staking_wrapper,
            FIRST_TOKEN_ID,
            0,
            &farm_in_amount,
            |sc| {
                let _ = sc.stake_farm_endpoint(OptionalValue::None);
            },
        )
        .assert_ok();

    farm_setup.b_mock.check_nft_balance::<Empty>(
        &first_user_addr,
        STAKING_FARM_TOKEN_ID,
        1,
        &farm_in_amount,
        None,
    );

    farm_setup
        .b_mock
        .execute_esdt_transfer(
            &first_user_addr,
            &auto_farm_wrapper,
            STAKING_FARM_TOKEN_ID,
            1,
            &farm_in_amount,
            |sc| {
                sc.call_deposit_farm_tokens();
            },
        )
        .assert_ok();

    farm_setup
        .b_mock
        .execute_tx(&second_user_addr, &auto_farm_wrapper, &rust_zero, |sc| {
            sc.register();
        })
        .assert_ok();

    farm_setup.set_user_energy(&first_user_addr, 1_000, 5, 500);
    farm_setup.set_user_energy(&second_user_addr, 9_000, 5, 500);

    // proxy claim for user - get registered
    farm_setup
        .b_mock
        .execute_tx(&proxy_address, &auto_farm_wrapper, &rust_zero, |sc| {
            let mut first_rew_wrapper = RewardsWrapper::new(managed_token_id!(LOCKED_TOKEN_ID));
            let mut second_rew_wrapper = RewardsWrapper::new(managed_token_id!(LOCKED_TOKEN_ID));

            sc.claim_fees_collector_rewards(
                &managed_address!(&first_user_addr),
                &mut first_rew_wrapper,
            );
            sc.claim_fees_collector_rewards(
                &managed_address!(&second_user_addr),
                &mut second_rew_wrapper,
            );

            sc.add_user_rewards(managed_address!(&first_user_addr), 1, first_rew_wrapper);
            sc.add_user_rewards(managed_address!(&second_user_addr), 2, second_rew_wrapper);
        })
        .assert_ok();

    // advance one week
    farm_setup.b_mock.set_block_epoch(8);
    farm_setup.b_mock.set_block_nonce(10);

    farm_setup
        .b_mock
        .execute_tx(&proxy_address, &auto_farm_wrapper, &rust_zero, |sc| {
            let mut mb_claim_args = MultiValueEncoded::new();
            mb_claim_args.push((managed_address!(&first_user_addr), ManagedVec::new()).into());
            sc.claim_all_rewards_and_compound(mb_claim_args);

            let accumulated_fees = sc.accumulated_fees().get();
            let mut expected_fees = MergedRewardsWrapper::<DebugApi> {
                opt_locked_tokens: None,
                other_tokens: UniquePayments::new(),
            };

            // values taken from fees collector test + farm_staking test
            let first_token_total =
                managed_biguint!(fees_collector_setup::USER_BALANCE) * 1_000u64 / 10_000u64 + 40u64;
            let second_token_total =
                managed_biguint!(fees_collector_setup::USER_BALANCE / 2u64) * 1_000u64 / 10_000u64;
            let locked_token_total = managed_biguint!(fees_collector_setup::USER_BALANCE / 100u64)
                * 1_000u64
                / 10_000u64;

            let first_expected_fee_amount = &first_token_total * FEE_PERCENTAGE / MAX_PERCENTAGE;
            let second_expected_fee_amount = &second_token_total * FEE_PERCENTAGE / MAX_PERCENTAGE;
            let expected_locked_fee_amount = &locked_token_total * FEE_PERCENTAGE / MAX_PERCENTAGE;

            expected_fees
                .other_tokens
                .add_payment(EsdtTokenPayment::new(
                    managed_token_id!(FIRST_TOKEN_ID),
                    0,
                    first_expected_fee_amount.clone(),
                ));

            expected_fees
                .other_tokens
                .add_payment(EsdtTokenPayment::new(
                    managed_token_id!(SECOND_TOKEN_ID),
                    0,
                    second_expected_fee_amount.clone(),
                ));

            expected_fees.opt_locked_tokens = Some(EsdtTokenPayment::new(
                managed_token_id!(LOCKED_TOKEN_ID),
                1,
                expected_locked_fee_amount.clone(),
            ));

            assert_eq!(accumulated_fees, expected_fees);

            // check user rewards
            // first token was compouned, so no rewards here
            let user_rewards = sc.get_user_rewards_view(managed_address!(&first_user_addr));
            let mut expected_user_rewards = MergedRewardsWrapper::<DebugApi> {
                opt_locked_tokens: None,
                other_tokens: UniquePayments::new(),
            };

            expected_user_rewards
                .other_tokens
                .add_payment(EsdtTokenPayment::new(
                    managed_token_id!(SECOND_TOKEN_ID),
                    0,
                    second_token_total - second_expected_fee_amount,
                ));

            expected_user_rewards.opt_locked_tokens = Some(EsdtTokenPayment::new(
                managed_token_id!(LOCKED_TOKEN_ID),
                1,
                locked_token_total - expected_locked_fee_amount,
            ));

            assert_eq!(user_rewards, expected_user_rewards);

            let actual_farm_staking_tokens = sc.user_farm_tokens(1).get();
            let expected_farm_staking_tokens = ManagedVec::from_single_item(EsdtTokenPayment::new(
                managed_token_id!(STAKING_FARM_TOKEN_ID),
                3,
                elrond_wasm::types::BigUint::from(farm_in_amount) + first_token_total
                    - first_expected_fee_amount,
            ));
            assert_eq!(actual_farm_staking_tokens, expected_farm_staking_tokens);
        })
        .assert_ok();
}
