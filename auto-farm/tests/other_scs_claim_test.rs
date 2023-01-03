pub mod farm_with_locked_rewards_setup;
pub mod metabonding_setup;

use crate::farm_with_locked_rewards_setup::FarmSetup;
use auto_farm::{
    common_storage::MAX_PERCENTAGE,
    fees::FeesModule,
    metabonding_actions::MetabondingActionsModule,
    user_rewards::{RewardsWrapper, UniquePayments, UserRewardsModule},
    AutoFarm,
};
use elrond_wasm::types::{EsdtTokenPayment, MultiValueEncoded};
use elrond_wasm_debug::{
    managed_address, managed_biguint, managed_token_id, rust_biguint,
    testing_framework::BlockchainStateWrapper, DebugApi,
};
use metabonding_setup::*;
use sc_whitelist_module::SCWhitelistModule;

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

    let mut b_mock = BlockchainStateWrapper::new();
    let rust_zero = rust_biguint!(0);

    let mb_setup = setup_metabonding(&mut b_mock, metabonding::contract_obj);

    let owner = b_mock.create_user_account(&rust_zero);
    let proxy_address = b_mock.create_user_account(&rust_zero);
    let auto_farm_wrapper = b_mock.create_sc_account(
        &rust_zero,
        Some(&owner),
        auto_farm::contract_obj,
        "auto farm",
    );

    b_mock
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

    // whitelist auto-farm SC in metabonding
    b_mock
        .execute_tx(&owner, &mb_setup, &rust_zero, |sc| {
            sc.sc_whitelist_addresses()
                .add(&managed_address!(auto_farm_wrapper.address_ref()))
        })
        .assert_ok();

    // proxy claim metabonding rewards for user
    // claim first 2 weeks
    let sig_first_user_week_1 = hex_literal::hex!("d47c0d67b2d25de8b4a3f43d91a2b5ccb522afac47321ae80bf89c90a4445b26adefa693ab685fa20891f736d74eb2dedc11c4b1a8d6e642fa28df270d6ebe08");
    let sig_first_user_week_2 = hex_literal::hex!("b4aadf08eea4cc7c636922511943edbab2ff6ef2558528e0e7b03c7448367989fe860ac091be4d942304f04c86b1eaa0501f36e02819a3c628b4c53f3d3ac801");

    b_mock
        .execute_tx(&proxy_address, &auto_farm_wrapper, &rust_zero, |sc| {
            let mut claim_args = MultiValueEncoded::new();
            claim_args.push(
                (
                    1usize,
                    managed_biguint!(25_000),
                    managed_biguint!(0),
                    (&sig_first_user_week_1).into(),
                )
                    .into(),
            );
            claim_args.push(
                (
                    2usize,
                    managed_biguint!(25_000),
                    managed_biguint!(0),
                    (&sig_first_user_week_2).into(),
                )
                    .into(),
            );

            sc.claim_metabonding_rewards(managed_address!(&farm_setup.first_user), claim_args);

            // taken from metabonding test
            let total_rewards_week1 = managed_biguint!(83_333_333 + 41_666_666);
            let total_rewards_week2 = managed_biguint!(50_000_000);

            // check fees
            let accumulated_fees = sc.accumulated_fees().get();
            let mut expected_fees = RewardsWrapper::<DebugApi> {
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

            // check user rewards
            let user_rewards = sc.get_user_rewards_view(managed_address!(&farm_setup.first_user));
            let mut expected_user_rewards = RewardsWrapper::<DebugApi> {
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
