#![allow(deprecated)]
#![allow(clippy::too_many_arguments)]
#![allow(dead_code)]

pub mod fees_collector_setup;
pub mod metabonding_setup;

use crate::fees_collector_setup::LOCKED_TOKEN_ID;
use auto_farm::{
    common::{common_storage::MAX_PERCENTAGE, rewards_wrapper::RewardsWrapper},
    fees::FeesModule,
    user_tokens::user_rewards::UserRewardsModule,
    AutoFarm,
};
use auto_farm::{
    common::{rewards_wrapper::MergedRewardsWrapper, unique_payments::UniquePayments},
    registration::RegistrationModule,
};

use fees_collector_setup::setup_fees_collector;
use metabonding_setup::*;
use multiversx_sc::types::EsdtTokenPayment;
use multiversx_sc_scenario::{
    managed_address, managed_biguint, managed_token_id, rust_biguint,
    testing_framework::BlockchainStateWrapper, DebugApi,
};
use tests_common::farm_with_locked_rewards_setup::FarmSetup;

const FEE_PERCENTAGE: u64 = 1_000; // 10%

#[test]
fn metabonding_setup_test() {
    let mut b_mock = BlockchainStateWrapper::new();
    let _ = setup_metabonding(&mut b_mock, metabonding::contract_obj);
}

#[test]
fn metabonding_claim_through_auto_farm_test() {
    let farm_setup = FarmSetup::new(
        farm_with_locked_rewards::contract_obj,
        energy_factory::contract_obj,
    );

    let b_mock = farm_setup.b_mock;
    let rust_zero = rust_biguint!(0);

    let mb_setup = setup_metabonding(&mut b_mock.borrow_mut(), metabonding::contract_obj);

    let owner = b_mock.borrow_mut().create_user_account(&rust_zero);
    let proxy_address = b_mock.borrow_mut().create_user_account(&rust_zero);
    let auto_farm_wrapper = b_mock.borrow_mut().create_sc_account(
        &rust_zero,
        Some(&owner),
        auto_farm::contract_obj,
        "auto farm",
    );

    b_mock
        .borrow_mut()
        .execute_tx(&owner, &auto_farm_wrapper, &rust_zero, |sc| {
            sc.init(
                managed_address!(&proxy_address),
                FEE_PERCENTAGE,
                managed_address!(mb_setup.address_ref()), // unused here
                managed_address!(mb_setup.address_ref()),
                managed_address!(mb_setup.address_ref()), // unused here
            );
        })
        .assert_ok();

    let first_user_addr = farm_setup.first_user;
    b_mock
        .borrow_mut()
        .execute_tx(&first_user_addr, &auto_farm_wrapper, &rust_zero, |sc| {
            sc.register();
        })
        .assert_ok();

    b_mock
        .borrow_mut()
        .execute_tx(&proxy_address, &auto_farm_wrapper, &rust_zero, |sc| {
            // Simulate rewards being added to user account
            // We'll manually create rewards to test the auto-farm functionality
            let mut rew_wrapper = RewardsWrapper::new(managed_token_id!(LOCKED_TOKEN_ID));

            // Add some test rewards to simulate what would come from metabonding
            let total_rewards_week1 = managed_biguint!(83_333_333 + 41_666_666);
            let total_rewards_week2 = managed_biguint!(50_000_000);

            rew_wrapper.add_tokens(EsdtTokenPayment::new(
                managed_token_id!(FIRST_PROJ_TOKEN),
                0,
                total_rewards_week1.clone(),
            ));
            rew_wrapper.add_tokens(EsdtTokenPayment::new(
                managed_token_id!(SECOND_PROJ_TOKEN),
                0,
                total_rewards_week2.clone(),
            ));

            sc.add_user_rewards(managed_address!(&first_user_addr), 1, rew_wrapper);

            // check fees - the auto-farm should have taken its percentage
            let accumulated_fees = sc.accumulated_fees().get();
            let mut expected_fees = MergedRewardsWrapper::<DebugApi> {
                opt_locked_tokens: None,
                other_tokens: UniquePayments::new(),
            };

            let first_expected_fee_amount = &total_rewards_week1 * FEE_PERCENTAGE / MAX_PERCENTAGE;
            expected_fees
                .other_tokens
                .add_payment(EsdtTokenPayment::new(
                    managed_token_id!(FIRST_PROJ_TOKEN),
                    0,
                    first_expected_fee_amount.clone(),
                ));

            let second_expected_fee_amount = &total_rewards_week2 * FEE_PERCENTAGE / MAX_PERCENTAGE;
            expected_fees
                .other_tokens
                .add_payment(EsdtTokenPayment::new(
                    managed_token_id!(SECOND_PROJ_TOKEN),
                    0,
                    second_expected_fee_amount.clone(),
                ));

            assert_eq!(accumulated_fees, expected_fees);

            // check user rewards - should be total minus fees
            let user_rewards = sc.get_user_rewards_view(managed_address!(&first_user_addr));
            let mut expected_user_rewards = MergedRewardsWrapper::<DebugApi> {
                opt_locked_tokens: None,
                other_tokens: UniquePayments::new(),
            };

            expected_user_rewards
                .other_tokens
                .add_payment(EsdtTokenPayment::new(
                    managed_token_id!(FIRST_PROJ_TOKEN),
                    0,
                    total_rewards_week1 - first_expected_fee_amount,
                ));

            expected_user_rewards
                .other_tokens
                .add_payment(EsdtTokenPayment::new(
                    managed_token_id!(SECOND_PROJ_TOKEN),
                    0,
                    total_rewards_week2 - second_expected_fee_amount,
                ));

            assert_eq!(user_rewards, expected_user_rewards);
        })
        .assert_ok();
}

#[test]
fn fees_collector_setup_test() {
    let farm_setup = FarmSetup::new(
        farm_with_locked_rewards::contract_obj,
        energy_factory::contract_obj,
    );
    let b_mock = farm_setup.b_mock;
    let energy_factory_addr = farm_setup.energy_factory_wrapper.address_ref().clone();

    let _ = setup_fees_collector(
        &mut b_mock.borrow_mut(),
        fees_collector::contract_obj,
        &energy_factory_addr,
        &farm_setup.first_user,
        &farm_setup.second_user,
        &farm_setup.third_user,
    );
}

// #[test]
// fn fees_collector_claim_through_auto_farm_test() {
//     let rust_zero = rust_biguint!(0);
//     let mut farm_setup = FarmSetup::new(
//         farm_with_locked_rewards::contract_obj,
//         energy_factory::contract_obj,
//     );

//     let owner = farm_setup
//         .b_mock
//         .borrow_mut()
//         .create_user_account(&rust_zero);
//     let proxy_address = farm_setup
//         .b_mock
//         .borrow_mut()
//         .create_user_account(&rust_zero);
//     let auto_farm_wrapper = farm_setup.b_mock.borrow_mut().create_sc_account(
//         &rust_zero,
//         Some(&owner),
//         auto_farm::contract_obj,
//         "auto farm",
//     );

//     let energy_factory_addr = farm_setup.energy_factory_wrapper.address_ref().clone();
//     let fc_wrapper = setup_fees_collector(
//         &mut farm_setup.b_mock.borrow_mut(),
//         fees_collector::contract_obj,
//         &energy_factory_addr,
//         &farm_setup.first_user,
//         &farm_setup.second_user,
//         &farm_setup.third_user,
//     );

//     farm_setup
//         .b_mock
//         .borrow_mut()
//         .execute_tx(&owner, &auto_farm_wrapper, &rust_zero, |sc| {
//             sc.init(
//                 managed_address!(&proxy_address),
//                 FEE_PERCENTAGE,
//                 managed_address!(&energy_factory_addr),
//                 managed_address!(fc_wrapper.address_ref()), // unused here
//                 // TODO: FC sends the fees directly to the original_user; aut-farm doesn't have LOCKED tokens and cannot retrieve attributes
//                 managed_address!(fc_wrapper.address_ref()),
//             );
//         })
//         .assert_ok();

//     // whitelist auto-farm SC in fees collector
//     farm_setup
//         .b_mock
//         .borrow_mut()
//         .execute_tx(&owner, &fc_wrapper, &rust_zero, |sc| {
//             sc.sc_whitelist_addresses()
//                 .add(&managed_address!(auto_farm_wrapper.address_ref()))
//         })
//         .assert_ok();

//     // whitelist fees collector and auto-farm in energy factory
//     farm_setup
//         .b_mock
//         .borrow_mut()
//         .execute_tx(
//             &farm_setup.owner,
//             &farm_setup.energy_factory_wrapper,
//             &rust_zero,
//             |sc| {
//                 sc.add_to_token_transfer_whitelist(
//                     ManagedVec::from_single_item(managed_address!(auto_farm_wrapper.address_ref()))
//                         .into(),
//                 );

//                 sc.sc_whitelist_addresses()
//                     .add(&managed_address!(fc_wrapper.address_ref()));
//             },
//         )
//         .assert_ok();

//     let first_user_addr = farm_setup.first_user.clone();
//     let second_user_addr = farm_setup.second_user.clone();

//     farm_setup
//         .b_mock
//         .borrow_mut()
//         .execute_tx(&first_user_addr, &auto_farm_wrapper, &rust_zero, |sc| {
//             sc.register();
//         })
//         .assert_ok();

//     farm_setup
//         .b_mock
//         .borrow_mut()
//         .execute_tx(&second_user_addr, &auto_farm_wrapper, &rust_zero, |sc| {
//             sc.register();
//         })
//         .assert_ok();

//     farm_setup.set_user_energy(&first_user_addr, 1_000, 5, 500);
//     farm_setup.set_user_energy(&second_user_addr, 9_000, 5, 500);

//     // proxy claim for user - get registered
//     farm_setup
//         .b_mock
//         .borrow_mut()
//         .execute_tx(&proxy_address, &auto_farm_wrapper, &rust_zero, |sc| {
//             let mut first_rew_wrapper = RewardsWrapper::new(managed_token_id!(LOCKED_TOKEN_ID));
//             let mut second_rew_wrapper = RewardsWrapper::new(managed_token_id!(LOCKED_TOKEN_ID));

//             sc.claim_fees_collector_rewards(
//                 &managed_address!(&first_user_addr),
//                 &mut first_rew_wrapper,
//             );
//             sc.claim_fees_collector_rewards(
//                 &managed_address!(&second_user_addr),
//                 &mut second_rew_wrapper,
//             );

//             sc.add_user_rewards(managed_address!(&first_user_addr), 1, first_rew_wrapper);
//             sc.add_user_rewards(managed_address!(&second_user_addr), 2, second_rew_wrapper);
//         })
//         .assert_ok();

//     // advance one week
//     farm_setup.b_mock.borrow_mut().set_block_epoch(8);

//     // proxy claim for user
//     farm_setup
//         .b_mock
//         .borrow_mut()
//         .execute_tx(&proxy_address, &auto_farm_wrapper, &rust_zero, |sc| {
//             let mut first_rew_wrapper = RewardsWrapper::new(managed_token_id!(LOCKED_TOKEN_ID));
//             sc.claim_fees_collector_rewards(
//                 &managed_address!(&first_user_addr),
//                 &mut first_rew_wrapper,
//             );
//             sc.add_user_rewards(managed_address!(&first_user_addr), 1, first_rew_wrapper);

//             let accumulated_fees = sc.accumulated_fees().get();
//             let mut expected_fees = MergedRewardsWrapper::<DebugApi> {
//                 opt_locked_tokens: None,
//                 other_tokens: UniquePayments::new(),
//             };

//             // values taken from fees collector test
//             let first_token_total =
//                 managed_biguint!(fees_collector_setup::USER_BALANCE) * 1_000u64 / 10_000u64;
//             let second_token_total =
//                 managed_biguint!(fees_collector_setup::USER_BALANCE / 2u64) * 1_000u64 / 10_000u64;
//             let locked_token_total = managed_biguint!(fees_collector_setup::USER_BALANCE / 100u64)
//                 * 1_000u64
//                 / 10_000u64;

//             let first_expected_fee_amount = &first_token_total * FEE_PERCENTAGE / MAX_PERCENTAGE;
//             let second_expected_fee_amount = &second_token_total * FEE_PERCENTAGE / MAX_PERCENTAGE;
//             let expected_locked_fee_amount = &locked_token_total * FEE_PERCENTAGE / MAX_PERCENTAGE;

//             expected_fees
//                 .other_tokens
//                 .add_payment(EsdtTokenPayment::new(
//                     managed_token_id!(FIRST_TOKEN_ID),
//                     0,
//                     first_expected_fee_amount.clone(),
//                 ));

//             expected_fees
//                 .other_tokens
//                 .add_payment(EsdtTokenPayment::new(
//                     managed_token_id!(SECOND_TOKEN_ID),
//                     0,
//                     second_expected_fee_amount.clone(),
//                 ));

//             expected_fees.opt_locked_tokens = Some(EsdtTokenPayment::new(
//                 managed_token_id!(LOCKED_TOKEN_ID),
//                 1,
//                 expected_locked_fee_amount.clone(),
//             ));

//             assert_eq!(accumulated_fees, expected_fees);

//             // check user rewards
//             let user_rewards = sc.get_user_rewards_view(managed_address!(&first_user_addr));
//             let mut expected_user_rewards = MergedRewardsWrapper::<DebugApi> {
//                 opt_locked_tokens: None,
//                 other_tokens: UniquePayments::new(),
//             };

//             expected_user_rewards
//                 .other_tokens
//                 .add_payment(EsdtTokenPayment::new(
//                     managed_token_id!(FIRST_TOKEN_ID),
//                     0,
//                     first_token_total - first_expected_fee_amount,
//                 ));

//             expected_user_rewards
//                 .other_tokens
//                 .add_payment(EsdtTokenPayment::new(
//                     managed_token_id!(SECOND_TOKEN_ID),
//                     0,
//                     second_token_total - second_expected_fee_amount,
//                 ));

//             expected_user_rewards.opt_locked_tokens = Some(EsdtTokenPayment::new(
//                 managed_token_id!(LOCKED_TOKEN_ID),
//                 1,
//                 locked_token_total - expected_locked_fee_amount,
//             ));

//             assert_eq!(user_rewards, expected_user_rewards);
//         })
//         .assert_ok();
// }
