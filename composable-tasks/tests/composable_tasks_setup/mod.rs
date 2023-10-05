#![allow(deprecated)]

use std::{cell::RefCell, rc::Rc};

use crate::{pair_setup::PairSetup, wegld_swap_setup::WegldSwapSetup};

use composable_tasks::{ComposableTasksContract, external_sc_interactions::wegld_swap::WegldSwapModule};
use multiversx_sc::{hex_literal, types::Address};
use multiversx_sc_scenario::{
    managed_biguint, rust_biguint,
    testing_framework::{BlockchainStateWrapper, ContractObjWrapper},
    DebugApi, managed_address,
};
use pair::safe_price::SafePriceModule;

pub static FARMING_TOKEN_ID: &[&[u8]] = &[b"LPTOK-123456", b"LPTOK-654321"];
pub static TOKEN_IDS: &[&[u8]] = &[b"FIRST-123456", b"SECOND-123456", b"THIRD-123456"];
pub static LP_TOKEN_IDS: &[&[u8]] = &[FARMING_TOKEN_ID[0], FARMING_TOKEN_ID[1], b"LPTHIRD-123456"];

pub struct ComposableTasksSetup<PairBuilder, WegldSwapBuilder, ComposableTasksBuilder>
where
    PairBuilder: 'static + Copy + Fn() -> pair::ContractObj<DebugApi>,
    WegldSwapBuilder: 'static + Copy + Fn() -> multiversx_wegld_swap_sc::ContractObj<DebugApi>,
    ComposableTasksBuilder: 'static + Copy + Fn() -> composable_tasks::ContractObj<DebugApi>,
{
    pub b_mock: Rc<RefCell<BlockchainStateWrapper>>,
    pub owner: Address,
    pub first_user: Address,
    pub second_user: Address,
    pub pair_setups: Vec<PairSetup<PairBuilder>>,
    pub wegld_swap_setup: WegldSwapSetup<WegldSwapBuilder>,
    pub ct_wrapper:
        ContractObjWrapper<composable_tasks::ContractObj<DebugApi>, ComposableTasksBuilder>,
}

impl<PairBuilder, WegldSwapBuilder, ComposableTasksBuilder>
    ComposableTasksSetup<PairBuilder, WegldSwapBuilder, ComposableTasksBuilder>
where
    PairBuilder: 'static + Copy + Fn() -> pair::ContractObj<DebugApi>,
    WegldSwapBuilder: 'static + Copy + Fn() -> multiversx_wegld_swap_sc::ContractObj<DebugApi>,
    ComposableTasksBuilder: 'static + Copy + Fn() -> composable_tasks::ContractObj<DebugApi>,
{
    // Pairs setup:
    // 3 pools (A, B), (A, C), (B, C),
    // A:B 1:2
    // A:C 1:6
    // B:C 1:3

    // Pools: (B = billion)
    // (A, B) => (1B, 2B)
    // (A, C) => (1B, 6B)
    // (B, C) => (1B, 3B)
    //
    // A_total = 2B
    // B_total = 3B
    // C_total = 9B
    pub fn new(
        pair_builder: PairBuilder,
        wegld_swap_builder: WegldSwapBuilder,
        ct_builder: ComposableTasksBuilder,
    ) -> Self {
        let rust_zero = rust_biguint!(0);
        let b_mock_new = BlockchainStateWrapper::new();
        let b_mock_ref = RefCell::new(b_mock_new);
        let b_mock_rc = Rc::new(b_mock_ref);
        let b_mock = b_mock_rc.clone();

        let owner = b_mock.borrow_mut().create_user_account(&rust_zero);
        let first_user = Address::from(hex_literal::hex!(
            "75736572315F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F"
        ));
        b_mock
            .borrow_mut()
            .create_user_account_fixed_address(&first_user, &rust_zero);

        // address:user2 from scenarios
        let second_user = Address::from(hex_literal::hex!(
            "75736572325F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F"
        ));
        b_mock
            .borrow_mut()
            .create_user_account_fixed_address(&second_user, &rust_zero);

        // setup composable tasks sc
        let ct_wrapper = b_mock.borrow_mut().create_sc_account(
            &rust_biguint!(0),
            Some(&owner),
            ct_builder,
            "composable tasks",
        );

        let wegld_swap_setup = WegldSwapSetup::new(b_mock.clone(), wegld_swap_builder, &owner);

        let mut first_pair_setup = PairSetup::new(
            b_mock.clone(),
            pair_builder,
            &owner,
            TOKEN_IDS[0],
            TOKEN_IDS[1],
            LP_TOKEN_IDS[0],
        );
        let mut second_pair_setup = PairSetup::new(
            b_mock.clone(),
            pair_builder,
            &owner,
            TOKEN_IDS[0],
            TOKEN_IDS[2],
            LP_TOKEN_IDS[1],
        );
        let mut third_pair_setup = PairSetup::new(
            b_mock.clone(),
            pair_builder,
            &owner,
            TOKEN_IDS[1],
            TOKEN_IDS[2],
            LP_TOKEN_IDS[2],
        );

        let first_token_total_amount = 2_000_000_000u64;
        let second_token_total_amount = 3_000_000_000u64;
        let third_token_total_amount = 9_000_000_000u64;

        b_mock.borrow_mut().set_esdt_balance(
            &owner,
            TOKEN_IDS[0],
            &rust_biguint!(first_token_total_amount),
        );
        b_mock.borrow_mut().set_esdt_balance(
            &owner,
            TOKEN_IDS[1],
            &rust_biguint!(second_token_total_amount),
        );
        b_mock.borrow_mut().set_esdt_balance(
            &owner,
            TOKEN_IDS[2],
            &rust_biguint!(third_token_total_amount),
        );

        let mut block_round: u64 = 1;
        b_mock.borrow_mut().set_block_round(block_round);

        // add initial liquidity
        first_pair_setup.add_liquidity(&owner, 1_000_000_000, 2_000_000_000);
        second_pair_setup.add_liquidity(&owner, 1_000_000_000, 6_000_000_000);
        third_pair_setup.add_liquidity(&owner, 1_000_000_000, 3_000_000_000);

        // setup price observations
        for _i in 1usize..=20 {
            block_round += 1;
            b_mock.borrow_mut().set_block_round(block_round);

            b_mock
                .borrow_mut()
                .execute_tx(
                    &owner,
                    &first_pair_setup.pair_wrapper,
                    &rust_biguint!(0),
                    |sc| {
                        sc.update_safe_price(
                            &managed_biguint!(1_000_000_000),
                            &managed_biguint!(2_000_000_000),
                        );
                    },
                )
                .assert_ok();

            b_mock
                .borrow_mut()
                .execute_tx(
                    &owner,
                    &second_pair_setup.pair_wrapper,
                    &rust_biguint!(0),
                    |sc| {
                        sc.update_safe_price(
                            &managed_biguint!(1_000_000_000),
                            &managed_biguint!(6_000_000_000),
                        );
                    },
                )
                .assert_ok();

            b_mock
                .borrow_mut()
                .execute_tx(
                    &owner,
                    &third_pair_setup.pair_wrapper,
                    &rust_biguint!(0),
                    |sc| {
                        sc.update_safe_price(
                            &managed_biguint!(1_000_000_000),
                            &managed_biguint!(3_000_000_000),
                        );
                    },
                )
                .assert_ok();
        }

        b_mock
            .borrow_mut()
            .execute_tx(&owner, &ct_wrapper, &rust_biguint!(0), |sc| {
                sc.init();

                let wegld_swap_addr: &Address = wegld_swap_setup.wegld_swap_wrapper.address_ref();
                sc.wrap_egld_addr().set(managed_address!(wegld_swap_addr));


                // TODO: Add to storage Pairs
            })
            .assert_ok();

        let pair_setups = vec![first_pair_setup, second_pair_setup, third_pair_setup];

        ComposableTasksSetup {
            b_mock,
            owner,
            first_user,
            second_user,
            pair_setups,
            wegld_swap_setup,
            ct_wrapper,
        }
    }
}