#![allow(deprecated)]

use auto_pos_creator::{
    external_sc_interactions::pair_actions::PairTokenPayments,
    multi_contract_interactions::{
        create_pos::{CreatePosModule, StepsToPerform},
        create_pos_endpoints::CreatePosEndpointsModule,
        exit_pos_endpoints::ExitPosEndpointsModule,
    },
};
use farm_staking::token_attributes::{StakingFarmTokenAttributes, UnbondSftAttributes};
use metastaking_setup::DUAL_YIELD_TOKEN_ID;
use multiversx_sc::{codec::Empty, types::EsdtTokenPayment};
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

#[test]
fn full_pos_creator_setup_test() {
    let _ = PosCreatorSetup::new(
        farm_with_locked_rewards::contract_obj,
        energy_factory::contract_obj,
        pair::contract_obj,
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
    let second_pair_addr = pos_creator_setup.pair_setups[2]
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
                let _ = sc.create_pos_from_single_token(
                    managed_address!(&second_pair_addr),
                    StepsToPerform::AddLiquidity,
                    1u32.into(),
                    1u32.into(),
                );
            },
        )
        .assert_ok();

    // bought B tokens with 100_000_000 A tokens
    // ratio in pair was A:B 1:2
    // ~200_000_000 B tokens received
    //
    // bought C tokens with 100_000_000 A tokens
    // ratio in pair was A:C 1:6
    // ~600_000_000 C received
    //
    // added liqudity with the received (B, C) tokens to the pool,
    // which had ratio of B:C 1:3
    // received 181_818_181 LP tokens
    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, LP_TOKEN_IDS[0], &rust_biguint!(0));
    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, LP_TOKEN_IDS[1], &rust_biguint!(0));
    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, LP_TOKEN_IDS[2], &rust_biguint!(181_818_181));

    // exit LP pos
    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &user_addr,
            &pos_creator_setup.pos_creator_wrapper,
            LP_TOKEN_IDS[2],
            0,
            &rust_biguint!(181_818_181),
            |sc| {
                let _ = sc.full_exit_pos_endpoint(1u32.into(), 1u32.into());
            },
        )
        .assert_ok();

    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, TOKEN_IDS[0], &rust_biguint!(0));
    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, TOKEN_IDS[1], &rust_biguint!(181_818_181));
    b_mock.borrow().check_esdt_balance(
        &user_addr,
        TOKEN_IDS[2],
        &rust_biguint!(3 * 181_818_181 + 2),
    );
}

#[test]
fn enter_lp_and_farm_through_pos_creator() {
    let pos_creator_setup = PosCreatorSetup::new(
        farm_with_locked_rewards::contract_obj,
        energy_factory::contract_obj,
        pair::contract_obj,
        farm_staking::contract_obj,
        farm_staking_proxy::contract_obj,
        auto_pos_creator::contract_obj,
    );
    let b_mock = pos_creator_setup.farm_setup.b_mock;

    let user_addr = pos_creator_setup.farm_setup.first_user;
    let user_second_token_balance = 200_000_000u64;
    b_mock.borrow_mut().set_esdt_balance(
        &user_addr,
        TOKEN_IDS[1],
        &rust_biguint!(user_second_token_balance),
    );

    // enter pair and farm from B tokens
    let second_pair_addr = pos_creator_setup.pair_setups[1]
        .pair_wrapper
        .address_ref()
        .clone();
    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &user_addr,
            &pos_creator_setup.pos_creator_wrapper,
            TOKEN_IDS[1],
            0,
            &rust_biguint!(user_second_token_balance),
            |sc| {
                let _ = sc.create_pos_from_single_token(
                    managed_address!(&second_pair_addr),
                    StepsToPerform::EnterFarm,
                    1u32.into(),
                    1u32.into(),
                );
            },
        )
        .assert_ok();

    // check user did not receive any LP tokens
    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, LP_TOKEN_IDS[0], &rust_biguint!(0));
    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, LP_TOKEN_IDS[1], &rust_biguint!(0));
    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, LP_TOKEN_IDS[2], &rust_biguint!(0));

    // check user received farm tokens
    // bought A tokens with 100_000_000 B tokens
    // pair had A:B ratio of 1:2
    // ~50_000_000 A tokens received
    //
    // bought C tokens with 100_000_000 B tokens
    // pair had B:C ratio of 1:3
    // ~300_000_000 THIRD tokens received
    //
    // added liquidty to (A, C pool) of (50M, 300M)
    // pool already had A:C ratio of 1:6
    // 45_454_545 LP tokens received
    b_mock.borrow().check_nft_balance::<Empty>(
        &user_addr,
        FARM_TOKEN_ID[1],
        1,
        &rust_biguint!(45_454_545),
        None,
    );

    // exit farm and then remove liquidity
    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &user_addr,
            &pos_creator_setup.pos_creator_wrapper,
            FARM_TOKEN_ID[1],
            1,
            &rust_biguint!(45_454_545),
            |sc| {
                let _ = sc.full_exit_pos_endpoint(1u32.into(), 1u32.into());
            },
        )
        .assert_ok();

    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, TOKEN_IDS[0], &rust_biguint!(47_164_502));
    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, TOKEN_IDS[1], &rust_biguint!(0));
    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, TOKEN_IDS[2], &rust_biguint!(270_000_000));
}

#[test]
fn enter_lp_farm_and_metastaking_through_pos_creator_test() {
    let pos_creator_setup = PosCreatorSetup::new(
        farm_with_locked_rewards::contract_obj,
        energy_factory::contract_obj,
        pair::contract_obj,
        farm_staking::contract_obj,
        farm_staking_proxy::contract_obj,
        auto_pos_creator::contract_obj,
    );
    let b_mock = pos_creator_setup.farm_setup.b_mock;

    let user_addr = pos_creator_setup.farm_setup.first_user;
    let user_third_token_balance = 200_000_000u64;
    b_mock.borrow_mut().set_esdt_balance(
        &user_addr,
        TOKEN_IDS[2],
        &rust_biguint!(user_third_token_balance),
    );

    b_mock.borrow_mut().set_block_round(21);

    // enter pair and farm from C tokens
    let first_pair_addr = pos_creator_setup.pair_setups[0]
        .pair_wrapper
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
                let _ = sc.create_pos_from_single_token(
                    managed_address!(&first_pair_addr),
                    StepsToPerform::EnterMetastaking,
                    1u32.into(),
                    1u32.into(),
                );
            },
        )
        .assert_ok();

    // check user did not receive any LP tokens
    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, LP_TOKEN_IDS[0], &rust_biguint!(0));
    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, LP_TOKEN_IDS[1], &rust_biguint!(0));
    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, LP_TOKEN_IDS[2], &rust_biguint!(0));

    // check user did not receive any farm tokens
    b_mock.borrow().check_nft_balance::<Empty>(
        &user_addr,
        FARM_TOKEN_ID[1],
        1,
        &rust_biguint!(0),
        None,
    );
    b_mock.borrow().check_nft_balance::<Empty>(
        &user_addr,
        FARM_TOKEN_ID[0],
        1,
        &rust_biguint!(0),
        None,
    );

    // check user received metastaking tokens
    b_mock.borrow().check_nft_balance::<Empty>(
        &user_addr,
        DUAL_YIELD_TOKEN_ID,
        1,
        &rust_biguint!(15_873_015),
        None,
    );

    // exit metastaking, farm and then remove liquidity
    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &user_addr,
            &pos_creator_setup.pos_creator_wrapper,
            DUAL_YIELD_TOKEN_ID,
            1,
            &rust_biguint!(15_873_015),
            |sc| {
                let _ = sc.full_exit_pos_endpoint(1u32.into(), 1u32.into());
            },
        )
        .assert_ok();

    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, TOKEN_IDS[0], &rust_biguint!(264_410));
    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, TOKEN_IDS[1], &rust_biguint!(31_935_484));
    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, TOKEN_IDS[2], &rust_biguint!(0));

    // check user has the unbond token for C tokens (i.e. staking tokens)
    b_mock.borrow().check_nft_balance(
        &user_addr,
        STAKING_FARM_TOKEN_ID,
        2,
        &rust_biguint!(15_967_742),
        Some(&UnbondSftAttributes { unlock_epoch: 5 }),
    );
}

#[test]
fn create_pos_from_two_tokens_balanced_ratio_test() {
    let pos_creator_setup = PosCreatorSetup::new(
        farm_with_locked_rewards::contract_obj,
        energy_factory::contract_obj,
        pair::contract_obj,
        farm_staking::contract_obj,
        farm_staking_proxy::contract_obj,
        auto_pos_creator::contract_obj,
    );

    let b_mock = pos_creator_setup.farm_setup.b_mock;

    // ratio for first pair is A:B 1:2
    let user_addr = pos_creator_setup.farm_setup.first_user;
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

    let first_pair_addr = pos_creator_setup.pair_setups[0]
        .pair_wrapper
        .address_ref()
        .clone();

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
        .execute_esdt_multi_transfer(
            &user_addr,
            &pos_creator_setup.pos_creator_wrapper,
            &payments,
            |sc| {
                let mut pair_payments = PairTokenPayments {
                    first_tokens: EsdtTokenPayment::new(
                        managed_token_id!(TOKEN_IDS[0]),
                        0,
                        managed_biguint!(user_first_token_balance),
                    ),
                    second_tokens: EsdtTokenPayment::new(
                        managed_token_id!(TOKEN_IDS[1]),
                        0,
                        managed_biguint!(user_second_token_balance),
                    ),
                };

                sc.balance_token_amounts_through_swaps(
                    managed_address!(&first_pair_addr),
                    &mut pair_payments,
                );

                // check nothing changed
                assert_eq!(pair_payments.first_tokens.amount, user_first_token_balance);
                assert_eq!(
                    pair_payments.second_tokens.amount,
                    user_second_token_balance
                );
            },
        )
        .assert_ok();
}

#[test]
fn create_pos_from_two_tokens_wrong_ratio() {
    let pos_creator_setup = PosCreatorSetup::new(
        farm_with_locked_rewards::contract_obj,
        energy_factory::contract_obj,
        pair::contract_obj,
        farm_staking::contract_obj,
        farm_staking_proxy::contract_obj,
        auto_pos_creator::contract_obj,
    );

    let b_mock = pos_creator_setup.farm_setup.b_mock;

    // ratio for first pair is A:B 1:2, try enter with 1:4 ratio
    let user_addr = pos_creator_setup.farm_setup.first_user;
    let user_first_token_balance = 100_000_000u64;
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

    let first_pair_addr = pos_creator_setup.pair_setups[0]
        .pair_wrapper
        .address_ref()
        .clone();

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
        .execute_esdt_multi_transfer(
            &user_addr,
            &pos_creator_setup.pos_creator_wrapper,
            &payments,
            |sc| {
                let mut pair_payments = PairTokenPayments {
                    first_tokens: EsdtTokenPayment::new(
                        managed_token_id!(TOKEN_IDS[0]),
                        0,
                        managed_biguint!(user_first_token_balance),
                    ),
                    second_tokens: EsdtTokenPayment::new(
                        managed_token_id!(TOKEN_IDS[1]),
                        0,
                        managed_biguint!(user_second_token_balance),
                    ),
                };

                sc.balance_token_amounts_through_swaps(
                    managed_address!(&first_pair_addr),
                    &mut pair_payments,
                );

                // check part of tokens was swapped to fix the ratio
                // initial was 100M A and 400M B
                assert_eq!(pair_payments.first_tokens.amount, 147_619_047);
                assert_eq!(pair_payments.second_tokens.amount, 300_000_000);
            },
        )
        .assert_ok();
}

#[test]
fn create_pos_from_lp_test() {
    let mut pos_creator_setup = PosCreatorSetup::new(
        farm_with_locked_rewards::contract_obj,
        energy_factory::contract_obj,
        pair::contract_obj,
        farm_staking::contract_obj,
        farm_staking_proxy::contract_obj,
        auto_pos_creator::contract_obj,
    );

    let b_mock = pos_creator_setup.farm_setup.b_mock;

    // ratio for first pair is A:B 1:2
    let user_addr = pos_creator_setup.farm_setup.first_user;
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

    pos_creator_setup.pair_setups[0].add_liquidity(
        &user_addr,
        user_first_token_balance,
        user_second_token_balance,
    );

    let lp_balance = 100_000_000u32;
    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, LP_TOKEN_IDS[0], &rust_biguint!(lp_balance));

    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &user_addr,
            &pos_creator_setup.pos_creator_wrapper,
            LP_TOKEN_IDS[0],
            0,
            &rust_biguint!(lp_balance),
            |sc| {
                sc.create_pos_from_lp(StepsToPerform::EnterMetastaking);
            },
        )
        .assert_ok();

    b_mock.borrow().check_nft_balance::<Empty>(
        &user_addr,
        DUAL_YIELD_TOKEN_ID,
        1,
        &rust_biguint!(90_909_090),
        None,
    );
}

#[test]
fn create_pos_with_farm_boosted_rewards_test() {
    let mut pos_creator_setup = PosCreatorSetup::new(
        farm_with_locked_rewards::contract_obj,
        energy_factory::contract_obj,
        pair::contract_obj,
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
    pos_creator_setup
        .farm_setup
        .set_user_energy(&user_addr, 1_000, 2, 1);
    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &user_addr,
            &pos_creator_setup.pos_creator_wrapper,
            LP_TOKEN_IDS[0],
            0,
            &rust_biguint!(lp_balance / 2),
            |sc| {
                sc.create_pos_from_lp(StepsToPerform::EnterFarm);
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
                sc.create_pos_from_lp(StepsToPerform::EnterMetastaking);
            },
        )
        .assert_ok();

    b_mock.borrow().check_nft_balance::<Empty>(
        &user_addr,
        DUAL_YIELD_TOKEN_ID,
        1,
        &rust_biguint!(45_454_545u64),
        None,
    );

    b_mock.borrow().check_nft_balance::<Empty>(
        &user_addr,
        LOCKED_REWARD_TOKEN_ID,
        1,
        &rust_biguint!(2_500u64),
        None,
    );
}

#[test]
fn enter_farm_staking_through_pos_creator_test() {
    let pos_creator_setup = PosCreatorSetup::new(
        farm_with_locked_rewards::contract_obj,
        energy_factory::contract_obj,
        pair::contract_obj,
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

    // user enter (B, C) pair with token A
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
                let (new_farm_token, _boosted_rewards_payment) = sc
                    .create_farm_staking_pos_from_single_token(managed_address!(
                        farm_staking_address
                    ))
                    .into_tuple();

                assert_eq!(
                    new_farm_token.token_identifier,
                    managed_token_id!(STAKING_FARM_TOKEN_ID)
                );
                assert_eq!(new_farm_token.token_nonce, 1);
                assert_eq!(new_farm_token.amount, expected_output_amount);
            },
        )
        .assert_ok();

    b_mock
        .borrow()
        .check_nft_balance::<StakingFarmTokenAttributes<DebugApi>>(
            &user_addr,
            STAKING_FARM_TOKEN_ID,
            1,
            &rust_biguint!(expected_output_amount),
            None,
        );
}
