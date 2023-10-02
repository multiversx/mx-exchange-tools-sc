#![allow(deprecated)]

use auto_farm::common::rewards_wrapper::{MergedRewardsWrapper, RewardsWrapper};
use auto_farm::common::{common_storage::MAX_PERCENTAGE, unique_payments::UniquePayments};
use common_structs::FarmTokenAttributes;
use energy_factory::energy::EnergyModule;
use energy_factory::locked_token_transfer::LockedTokenTransferModule;
use energy_query::Energy;
use multiversx_sc::codec::Empty;
use multiversx_sc::types::{BigInt, EsdtTokenPayment, ManagedVec, MultiValueEncoded};
use multiversx_sc_scenario::testing_framework::TxTokenTransfer;
use multiversx_sc_scenario::{
    managed_address, managed_biguint, managed_token_id, rust_biguint, DebugApi,
};
use sc_whitelist_module::SCWhitelistModule;
use simple_lock::locked_token::LockedTokenAttributes;

use auto_farm::external_sc_interactions::farm_actions::FarmActionsModule;
use auto_farm::fees::FeesModule;
use auto_farm::user_tokens::user_farm_tokens::UserFarmTokensModule;
use auto_farm::user_tokens::user_rewards::UserRewardsModule;
use auto_farm::whitelists::farms_whitelist::FarmsWhitelistModule;
use auto_farm::AutoFarm;

use tests_common::farm_with_locked_rewards_setup::{
    FarmSetup, FARM_TOKEN_ID, LOCKED_REWARD_TOKEN_ID,
};

const FEE_PERCENTAGE: u64 = 1_000; // 10%

const FIRST_FARM_INDEX: usize = 0;
const SECOND_FARM_INDEX: usize = 1;

#[test]
fn user_enter_and_claim_two_farms_test() {
    DebugApi::dummy();
    let mut farm_setup = FarmSetup::new(
        farm_with_locked_rewards::contract_obj,
        energy_factory::contract_obj,
    );

    farm_setup.b_mock.borrow_mut().set_block_epoch(2);
    let third_user = farm_setup.third_user.clone();

    // first enter farm
    let first_farm_token_amount = 100_000_000;
    let first_user = farm_setup.first_user.clone();
    farm_setup.set_user_energy(&first_user, 1_000, 2, 1);
    farm_setup.enter_farm(FIRST_FARM_INDEX, &first_user, first_farm_token_amount);

    // second enter farm
    let second_farm_token_amount = 50_000_000;
    farm_setup.enter_farm(SECOND_FARM_INDEX, &first_user, second_farm_token_amount);

    // advance blocks - 10 blocks - 10 * 1_000 = 10_000 total rewards
    // 7_500 base farm, 2_500 boosted yields
    farm_setup.b_mock.borrow_mut().set_block_nonce(10);

    // random tx on end of week 1, to cummulate rewards
    farm_setup.b_mock.borrow_mut().set_block_epoch(6);
    farm_setup.set_user_energy(&first_user, 1_000, 6, 1);
    farm_setup.set_user_energy(&third_user, 1, 6, 1);

    farm_setup.enter_farm(FIRST_FARM_INDEX, &third_user, 1);
    farm_setup.exit_farm(FIRST_FARM_INDEX, &third_user, 2, 1);
    farm_setup.enter_farm(SECOND_FARM_INDEX, &third_user, 1);
    farm_setup.exit_farm(SECOND_FARM_INDEX, &third_user, 2, 1);

    // advance 1 week
    farm_setup.b_mock.borrow_mut().set_block_epoch(10);
    farm_setup.set_user_energy(&first_user, 1_000, 10, 1);

    // first user claim - 75% of 10_000, 2_500 reserved for boosted yields
    let first_base_farm_amt = 7_500;

    // Boosted yields rewards formula
    // total_boosted_rewards * (energy_const * user_energy / total_energy + farm_const * user_farm / total_farm) / (energy_const + farm_const)
    let first_boosted_amt = 2_500;
    let first_total = first_base_farm_amt + first_boosted_amt;

    let first_receveived_reward_amt =
        farm_setup.claim_rewards(FIRST_FARM_INDEX, &first_user, 1, first_farm_token_amount);
    assert_eq!(first_receveived_reward_amt, first_total);

    farm_setup
        .b_mock
        .borrow_mut()
        .check_nft_balance::<FarmTokenAttributes<DebugApi>>(
            &first_user,
            FARM_TOKEN_ID[FIRST_FARM_INDEX],
            3,
            &rust_biguint!(first_farm_token_amount),
            None,
        );

    farm_setup
        .b_mock
        .borrow_mut()
        .check_nft_balance::<LockedTokenAttributes<DebugApi>>(
            &first_user,
            LOCKED_REWARD_TOKEN_ID,
            1,
            &rust_biguint!(first_receveived_reward_amt),
            None,
        );

    // second farm claim
    let second_receveived_reward_amt =
        farm_setup.claim_rewards(SECOND_FARM_INDEX, &first_user, 1, second_farm_token_amount);
    assert_eq!(second_receveived_reward_amt, first_total);

    farm_setup
        .b_mock
        .borrow_mut()
        .check_nft_balance::<FarmTokenAttributes<DebugApi>>(
            &first_user,
            FARM_TOKEN_ID[SECOND_FARM_INDEX],
            3,
            &rust_biguint!(second_farm_token_amount),
            None,
        );

    farm_setup
        .b_mock
        .borrow_mut()
        .check_nft_balance::<LockedTokenAttributes<DebugApi>>(
            &first_user,
            LOCKED_REWARD_TOKEN_ID,
            1,
            &rust_biguint!(first_receveived_reward_amt + second_receveived_reward_amt),
            None,
        );
}

#[test]
fn claim_rewards_through_auto_farm() {
    DebugApi::dummy();
    let mut farm_setup = FarmSetup::new(
        farm_with_locked_rewards::contract_obj,
        energy_factory::contract_obj,
    );

    farm_setup.b_mock.borrow_mut().set_block_epoch(2);

    // setup auto-farm SC
    let rust_zero = rust_biguint!(0);
    let proxy_address = farm_setup
        .b_mock
        .borrow_mut()
        .create_user_account(&rust_zero);
    let auto_farm_wrapper = farm_setup.b_mock.borrow_mut().create_sc_account(
        &rust_zero,
        Some(&farm_setup.owner),
        auto_farm::contract_obj,
        "auto farm",
    );
    let energy_factory_addr = farm_setup.energy_factory_wrapper.address_ref().clone();
    let mut farms = Vec::new();
    for farm_wrapper in &farm_setup.farm_wrappers {
        farms.push(farm_wrapper.address_ref().clone());
    }

    farm_setup
        .b_mock
        .borrow_mut()
        .execute_tx(&farm_setup.owner, &auto_farm_wrapper, &rust_zero, |sc| {
            sc.init(
                managed_address!(&proxy_address),
                FEE_PERCENTAGE,
                managed_address!(&energy_factory_addr),
                managed_address!(&energy_factory_addr), // unused here
                managed_address!(&energy_factory_addr), // unused here
            );

            let mut args = MultiValueEncoded::new();
            for farm in &farms {
                args.push(managed_address!(farm));
            }
            sc.add_farms(args);
        })
        .assert_ok();

    // whitelist auto-farm SC in farms
    for farm_wrapper in &farm_setup.farm_wrappers {
        farm_setup
            .b_mock
            .borrow_mut()
            .execute_tx(&farm_setup.owner, farm_wrapper, &rust_zero, |sc| {
                sc.add_sc_address_to_whitelist(managed_address!(auto_farm_wrapper.address_ref()));
            })
            .assert_ok();
    }

    // whitelist auto-farm SC in energy factory
    farm_setup
        .b_mock
        .borrow_mut()
        .execute_tx(
            &farm_setup.owner,
            &farm_setup.energy_factory_wrapper,
            &rust_zero,
            |sc| {
                sc.add_to_token_transfer_whitelist(
                    ManagedVec::from_single_item(managed_address!(auto_farm_wrapper.address_ref()))
                        .into(),
                );
            },
        )
        .assert_ok();

    let third_user = farm_setup.third_user.clone();

    // first enter farm
    let first_farm_token_amount = 100_000_000;
    let first_user = farm_setup.first_user.clone();
    farm_setup.set_user_energy(&first_user, 1_000, 2, 1);
    farm_setup.enter_farm(FIRST_FARM_INDEX, &first_user, first_farm_token_amount);

    // second enter farm
    let second_farm_token_amount = 50_000_000;
    farm_setup.enter_farm(SECOND_FARM_INDEX, &first_user, second_farm_token_amount);

    // advance blocks - 10 blocks - 10 * 1_000 = 10_000 total rewards
    // 7_500 base farm, 2_500 boosted yields
    farm_setup.b_mock.borrow_mut().set_block_nonce(10);

    // random tx on end of week 1, to cummulate rewards
    farm_setup.b_mock.borrow_mut().set_block_epoch(6);
    farm_setup.set_user_energy(&first_user, 1_000, 6, 1);
    farm_setup.set_user_energy(&third_user, 1, 6, 1);

    farm_setup.enter_farm(FIRST_FARM_INDEX, &third_user, 1);
    farm_setup.exit_farm(FIRST_FARM_INDEX, &third_user, 2, 1);
    farm_setup.enter_farm(SECOND_FARM_INDEX, &third_user, 1);
    farm_setup.exit_farm(SECOND_FARM_INDEX, &third_user, 2, 1);

    // advance 1 week
    farm_setup.b_mock.borrow_mut().set_block_epoch(10);
    farm_setup.set_user_energy(&first_user, 1_000, 10, 1);

    // user deposit farm tokens
    let farm_tokens = [
        TxTokenTransfer {
            token_identifier: FARM_TOKEN_ID[FIRST_FARM_INDEX].to_vec(),
            nonce: 1,
            value: rust_biguint!(first_farm_token_amount),
        },
        TxTokenTransfer {
            token_identifier: FARM_TOKEN_ID[SECOND_FARM_INDEX].to_vec(),
            nonce: 1,
            value: rust_biguint!(second_farm_token_amount),
        },
    ];
    farm_setup
        .b_mock
        .borrow_mut()
        .execute_esdt_multi_transfer(&first_user, &auto_farm_wrapper, &farm_tokens, |sc| {
            sc.deposit_farm_tokens();
        })
        .assert_ok();

    // proxy claim in user's place
    let total_expected_rewards = 20_000; // taken from the other test
    let expected_fee_amount = total_expected_rewards * FEE_PERCENTAGE / MAX_PERCENTAGE;
    let expected_user_rewards_amount = total_expected_rewards - expected_fee_amount;
    farm_setup
        .b_mock
        .borrow_mut()
        .execute_tx(&proxy_address, &auto_farm_wrapper, &rust_zero, |sc| {
            let mut rew_wrapper = RewardsWrapper::new(managed_token_id!(LOCKED_REWARD_TOKEN_ID));
            sc.claim_all_farm_rewards(&managed_address!(&first_user), 1, &mut rew_wrapper);
            sc.add_user_rewards(managed_address!(&first_user), 1, rew_wrapper);

            // check new user farm tokens
            let user_farm_tokens = sc.get_user_farm_tokens_view(managed_address!(&first_user));
            let mut expected_user_farm_tokens = ManagedVec::new();
            expected_user_farm_tokens.push(EsdtTokenPayment::new(
                managed_token_id!(FARM_TOKEN_ID[FIRST_FARM_INDEX]),
                3,
                managed_biguint!(first_farm_token_amount),
            ));
            expected_user_farm_tokens.push(EsdtTokenPayment::new(
                managed_token_id!(FARM_TOKEN_ID[SECOND_FARM_INDEX]),
                3,
                managed_biguint!(second_farm_token_amount),
            ));
            assert_eq!(user_farm_tokens, expected_user_farm_tokens);

            // check user rewards
            let user_rewards = sc.get_user_rewards_view(managed_address!(&first_user));
            let expected_user_rewards = MergedRewardsWrapper::<DebugApi> {
                opt_locked_tokens: Some(EsdtTokenPayment::new(
                    managed_token_id!(LOCKED_REWARD_TOKEN_ID),
                    1,
                    managed_biguint!(expected_user_rewards_amount),
                )),
                other_tokens: UniquePayments::new(),
            };
            assert_eq!(user_rewards, expected_user_rewards);

            // check fees
            let accumulated_fees = sc.accumulated_fees().get();
            let expected_fees = MergedRewardsWrapper::<DebugApi> {
                opt_locked_tokens: Some(EsdtTokenPayment::new(
                    managed_token_id!(LOCKED_REWARD_TOKEN_ID),
                    1,
                    managed_biguint!(expected_fee_amount),
                )),
                other_tokens: UniquePayments::new(),
            };
            assert_eq!(accumulated_fees, expected_fees);
        })
        .assert_ok();

    // check energy is updated accordingly
    farm_setup
        .b_mock
        .borrow_mut()
        .execute_query(&farm_setup.energy_factory_wrapper, |sc| {
            let first_user_energy = sc.user_energy(&managed_address!(&first_user)).get();
            // unlock epoch for new tokens = 10 + 4 * 365 = 1470
            let expected_first_user_energy = Energy::new(
                BigInt::from(managed_biguint!(
                    1_000u64 + expected_user_rewards_amount * (1_470 - 10)
                )),
                10,
                managed_biguint!(expected_user_rewards_amount) + 1u64, // user had 1 token
            );
            assert_eq!(first_user_energy, expected_first_user_energy);

            // check proxy address energy
            let proxy_energy = sc.user_energy(&managed_address!(&proxy_address)).get();
            let expected_proxy_energy = Energy::new(
                BigInt::from(managed_biguint!(expected_fee_amount * (1_470 - 10))),
                10,
                managed_biguint!(expected_fee_amount),
            );
            assert_eq!(proxy_energy, expected_proxy_energy);
        })
        .assert_ok();
}

#[test]
fn withdraw_specific_farm_tokens_test() {
    DebugApi::dummy();
    let mut farm_setup = FarmSetup::new(
        farm_with_locked_rewards::contract_obj,
        energy_factory::contract_obj,
    );

    farm_setup.b_mock.borrow_mut().set_block_epoch(2);

    // setup auto-farm SC
    let rust_zero = rust_biguint!(0);
    let proxy_address = farm_setup
        .b_mock
        .borrow_mut()
        .create_user_account(&rust_zero);
    let auto_farm_wrapper = farm_setup.b_mock.borrow_mut().create_sc_account(
        &rust_zero,
        Some(&farm_setup.owner),
        auto_farm::contract_obj,
        "auto farm",
    );
    let energy_factory_addr = farm_setup.energy_factory_wrapper.address_ref().clone();
    let mut farms = Vec::new();
    for farm_wrapper in &farm_setup.farm_wrappers {
        farms.push(farm_wrapper.address_ref().clone());
    }

    farm_setup
        .b_mock
        .borrow_mut()
        .execute_tx(&farm_setup.owner, &auto_farm_wrapper, &rust_zero, |sc| {
            sc.init(
                managed_address!(&proxy_address),
                FEE_PERCENTAGE,
                managed_address!(&energy_factory_addr),
                managed_address!(&energy_factory_addr), // unused here
                managed_address!(&energy_factory_addr), // unused here
            );

            let mut args = MultiValueEncoded::new();
            for farm in &farms {
                args.push(managed_address!(farm));
            }
            sc.add_farms(args);
        })
        .assert_ok();

    // whitelist auto-farm SC in farms
    for farm_wrapper in &farm_setup.farm_wrappers {
        farm_setup
            .b_mock
            .borrow_mut()
            .execute_tx(&farm_setup.owner, farm_wrapper, &rust_zero, |sc| {
                sc.add_sc_address_to_whitelist(managed_address!(auto_farm_wrapper.address_ref()));
            })
            .assert_ok();
    }

    // whitelist auto-farm SC in energy factory
    farm_setup
        .b_mock
        .borrow_mut()
        .execute_tx(
            &farm_setup.owner,
            &farm_setup.energy_factory_wrapper,
            &rust_zero,
            |sc| {
                sc.add_to_token_transfer_whitelist(
                    ManagedVec::from_single_item(managed_address!(auto_farm_wrapper.address_ref()))
                        .into(),
                );
            },
        )
        .assert_ok();

    // first enter farm
    let first_farm_token_amount = 100_000_000;
    let first_user = farm_setup.first_user.clone();
    farm_setup.set_user_energy(&first_user, 1_000, 2, 1);
    farm_setup.enter_farm(FIRST_FARM_INDEX, &first_user, first_farm_token_amount);

    // second enter farm
    let second_farm_token_amount = 50_000_000;
    farm_setup.enter_farm(SECOND_FARM_INDEX, &first_user, second_farm_token_amount);

    // user deposit farm tokens
    let farm_tokens = [
        TxTokenTransfer {
            token_identifier: FARM_TOKEN_ID[FIRST_FARM_INDEX].to_vec(),
            nonce: 1,
            value: rust_biguint!(first_farm_token_amount),
        },
        TxTokenTransfer {
            token_identifier: FARM_TOKEN_ID[SECOND_FARM_INDEX].to_vec(),
            nonce: 1,
            value: rust_biguint!(second_farm_token_amount),
        },
    ];
    farm_setup
        .b_mock
        .borrow_mut()
        .execute_esdt_multi_transfer(&first_user, &auto_farm_wrapper, &farm_tokens, |sc| {
            sc.deposit_farm_tokens();
        })
        .assert_ok();

    // user withdraw 1/2 of first token, and 1/4 of second token
    farm_setup
        .b_mock
        .borrow_mut()
        .execute_tx(&first_user, &auto_farm_wrapper, &rust_zero, |sc| {
            let mut tokens_to_withdraw = ManagedVec::new();
            tokens_to_withdraw.push(EsdtTokenPayment::new(
                managed_token_id!(FARM_TOKEN_ID[FIRST_FARM_INDEX]),
                1,
                managed_biguint!(first_farm_token_amount / 2),
            ));
            tokens_to_withdraw.push(EsdtTokenPayment::new(
                managed_token_id!(FARM_TOKEN_ID[SECOND_FARM_INDEX]),
                1,
                managed_biguint!(second_farm_token_amount / 4),
            ));

            sc.withdraw_specific_farm_tokens_endpoint(tokens_to_withdraw);

            // check remaining farm tokens storage
            let user_farm_tokens = sc.get_user_farm_tokens_view(managed_address!(&first_user));
            let mut expected_user_farm_tokens = ManagedVec::new();
            expected_user_farm_tokens.push(EsdtTokenPayment::new(
                managed_token_id!(FARM_TOKEN_ID[FIRST_FARM_INDEX]),
                1,
                managed_biguint!(first_farm_token_amount / 2),
            ));
            expected_user_farm_tokens.push(EsdtTokenPayment::new(
                managed_token_id!(FARM_TOKEN_ID[SECOND_FARM_INDEX]),
                1,
                managed_biguint!(second_farm_token_amount * 3 / 4),
            ));
            assert_eq!(user_farm_tokens, expected_user_farm_tokens);
        })
        .assert_ok();

    // check user received the tokens
    farm_setup.b_mock.borrow_mut().check_nft_balance::<Empty>(
        &first_user,
        FARM_TOKEN_ID[FIRST_FARM_INDEX],
        1,
        &rust_biguint!(first_farm_token_amount / 2),
        None,
    );
    farm_setup.b_mock.borrow_mut().check_nft_balance::<Empty>(
        &first_user,
        FARM_TOKEN_ID[SECOND_FARM_INDEX],
        1,
        &rust_biguint!(second_farm_token_amount / 4),
        None,
    );
}
