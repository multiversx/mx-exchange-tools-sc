use auto_pos_creator::multi_contract_interactions::{
    create_pos::CreatePosModule, exit_pos::ExitPosModule,
};
use elrond_wasm::elrond_codec::Empty;
use elrond_wasm_debug::{managed_address, rust_biguint};
use farm_staking::token_attributes::UnbondSftAttributes;
use metastaking_setup::DUAL_YIELD_TOKEN_ID;
use pos_creator_setup::{PosCreatorSetup, LP_TOKEN_IDS, TOKEN_IDS};
use tests_common::{
    farm_staking_setup::STAKING_FARM_TOKEN_ID, farm_with_locked_rewards_setup::FARM_TOKEN_ID,
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
    let b_mock = pos_creator_setup.farm_setup.b_mock.clone();

    let user_addr = pos_creator_setup.farm_setup.first_user.clone();
    let user_first_token_balance = 200_000_000u64;
    b_mock.borrow_mut().set_esdt_balance(
        &user_addr,
        TOKEN_IDS[0],
        &rust_biguint!(user_first_token_balance),
    );

    // user enter (second token, third token) pair with first token
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
                let _ = sc.create_pos_from_single_token(managed_address!(&second_pair_addr));
            },
        )
        .assert_ok();

    // bought SECOND tokens with 100_000_000 FIRST tokens
    // ratio in pair was FIRST:SECOND 2:1
    // ~50_000_000 SECOND received
    //
    // bought THIRD tokens with 100_000_000 FIRST tokens
    // ratio in pair was FIRST:THIRD 1:1
    // ~100_000_000 THIRD received
    //
    // added liqudity with the received (SECOND, THIRD) tokens, to pool,
    // which had ratio of SECOND:THIRD 1:2 (500_000_000, 1_000_000_000)
    // received 45_454_545 LP tokens
    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, LP_TOKEN_IDS[0], &rust_biguint!(0));
    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, LP_TOKEN_IDS[1], &rust_biguint!(0));
    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, LP_TOKEN_IDS[2], &rust_biguint!(45_454_545));

    // exit LP pos
    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &user_addr,
            &pos_creator_setup.pos_creator_wrapper,
            LP_TOKEN_IDS[2],
            0,
            &rust_biguint!(45_454_545),
            |sc| {
                let _ = sc.full_exit_pos();
            },
        )
        .assert_ok();

    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, TOKEN_IDS[0], &rust_biguint!(0));
    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, TOKEN_IDS[1], &rust_biguint!(45_454_545));
    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, TOKEN_IDS[2], &rust_biguint!(2 * 45_454_545));
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
    let b_mock = pos_creator_setup.farm_setup.b_mock.clone();

    let user_addr = pos_creator_setup.farm_setup.first_user.clone();
    let user_second_token_balance = 200_000_000u64;
    b_mock.borrow_mut().set_esdt_balance(
        &user_addr,
        TOKEN_IDS[1],
        &rust_biguint!(user_second_token_balance),
    );

    // enter pair and farm from SECOND tokens
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
                let _ = sc.create_pos_from_single_token(managed_address!(&second_pair_addr));
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
    // bought FIRST tokens with 100_000_000 SECOND tokens
    // pair had FIRST:SECOND ratio of 2:1
    // ~200_000_000 FIRST tokens received
    //
    // bought THIRD tokens with 100_000_000 SECOND tokens
    // ~200_000_000 THIRD tokens received
    //
    // added liquidty to (FIRST, THIRD pool) of (200M, 200M)
    // pool already had (1_000_000_000, 1_000_000_000)
    // 166_666_666 LP tokens received
    b_mock.borrow().check_nft_balance::<Empty>(
        &user_addr,
        FARM_TOKEN_ID[1],
        1,
        &rust_biguint!(166_666_666),
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
            &rust_biguint!(166_666_666),
            |sc| {
                let _ = sc.full_exit_pos();
            },
        )
        .assert_ok();

    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, TOKEN_IDS[0], &rust_biguint!(165_000_000));
    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, TOKEN_IDS[1], &rust_biguint!(0));
    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, TOKEN_IDS[2], &rust_biguint!(165_000_000));
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
    let b_mock = pos_creator_setup.farm_setup.b_mock.clone();

    let user_addr = pos_creator_setup.farm_setup.first_user.clone();
    let user_third_token_balance = 200_000_000u64;
    b_mock.borrow_mut().set_esdt_balance(
        &user_addr,
        TOKEN_IDS[2],
        &rust_biguint!(user_third_token_balance),
    );

    // enter pair and farm from SECOND tokens
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
                let _ = sc.create_pos_from_single_token(managed_address!(&first_pair_addr));
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
        &rust_biguint!(83_333_332),
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
            &rust_biguint!(83_333_332),
            |sc| {
                let _ = sc.full_exit_pos();
            },
        )
        .assert_ok();

    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, TOKEN_IDS[0], &rust_biguint!(0));
    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, TOKEN_IDS[1], &rust_biguint!(45_000_000));
    b_mock
        .borrow()
        .check_esdt_balance(&user_addr, TOKEN_IDS[2], &rust_biguint!(0));

    // check user has the unbond token for THIRD tokens (i.e. staking tokens)
    b_mock.borrow().check_nft_balance(
        &user_addr,
        STAKING_FARM_TOKEN_ID,
        2,
        &rust_biguint!(2 * 45_000_000),
        Some(&UnbondSftAttributes { unlock_epoch: 5 }),
    );
}
