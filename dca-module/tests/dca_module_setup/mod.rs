#![allow(deprecated)]

use std::{cell::RefCell, rc::Rc};

use crate::pair_setup::PairSetup;
use crate::router_setup::RouterSetup;

use dca_module::{
    user_data::{
        action::{
            action_types::{ActionId, Timestamp, TotalActions, TradeFrequency},
            user_action::ActionModule,
        },
        funds::FundsModule,
    },
    DcaModule,
};
use multiversx_sc::types::{Address, BigUint, MultiValueEncoded};
use multiversx_sc_modules::pause::PauseModule;
use multiversx_sc_scenario::{
    imports::{BlockchainStateWrapper, TxResult, TxTokenTransfer},
    managed_address, managed_biguint, managed_token_id, rust_biguint,
    testing_framework::ContractObjWrapper,
    DebugApi,
};

use pair::safe_price::SafePriceModule;

pub static TOKEN_IDS: &[&[u8]] = &[b"FIRST-123456", b"SECOND-123456", b"THIRD-123456"];
pub static LP_TOKEN_IDS: &[&[u8]] = &[b"LPFIRST-123456", b"LPSECOND-123456", b"LPTHIRD-123456"];
pub static WEGLD_TOKEN_ID: &[u8] = b"WEGLD-123456";
pub static DUAL_YIELD_TOKEN_ID: &[u8] = b"DUALYIELD-123456";

pub const START_TIME: Timestamp = 700_000_000; // random timestamp

#[derive(Clone)]
pub struct DummySwapArgs {
    pub pair_addr: Address,
    pub requested_token: Vec<u8>,
    pub min_amount_out: u64,
}

pub struct DcaModuleSetup<PairBuilder, RouterBuilder, DcaModuleBuilder>
where
    PairBuilder: 'static + Copy + Fn() -> pair::ContractObj<DebugApi>,
    RouterBuilder: 'static + Copy + Fn() -> router::ContractObj<DebugApi>,
    DcaModuleBuilder: 'static + Copy + Fn() -> dca_module::ContractObj<DebugApi>,
{
    pub b_mock: Rc<RefCell<BlockchainStateWrapper>>,
    pub owner: Address,
    pub user: Address,
    pub current_time: Timestamp,
    pub pair_setups: Vec<PairSetup<PairBuilder>>,
    pub router_setup: RouterSetup<RouterBuilder>,
    pub dca_module_wrapper: ContractObjWrapper<dca_module::ContractObj<DebugApi>, DcaModuleBuilder>,
}

impl<PairBuilder, RouterBuilder, DcaModuleBuilder>
    DcaModuleSetup<PairBuilder, RouterBuilder, DcaModuleBuilder>
where
    PairBuilder: 'static + Copy + Fn() -> pair::ContractObj<DebugApi>,
    RouterBuilder: 'static + Copy + Fn() -> router::ContractObj<DebugApi>,
    DcaModuleBuilder: 'static + Copy + Fn() -> dca_module::ContractObj<DebugApi>,
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
        router_builder: RouterBuilder,
        dca_module_builder: DcaModuleBuilder,
    ) -> Self {
        let b_mock_simple = BlockchainStateWrapper::new();
        let b_mock_ref = RefCell::new(b_mock_simple);
        let b_mock = Rc::new(b_mock_ref);

        let owner = b_mock.borrow_mut().create_user_account(&rust_biguint!(0));
        let user = b_mock.borrow_mut().create_user_account(&rust_biguint!(0));
        b_mock
            .borrow_mut()
            .set_esdt_balance(&user, TOKEN_IDS[0], &rust_biguint!(5_000));

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
        let mut router_setup = RouterSetup::new(
            b_mock.clone(),
            router_builder,
            &owner,
            first_pair_setup.pair_wrapper.address_ref(),
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

        // whitelist pairs in router
        router_setup.whitelist_pair(
            &owner,
            &first_pair_setup.first_token_id,
            &first_pair_setup.second_token_id,
            first_pair_setup.pair_wrapper.address_ref(),
        );
        router_setup.whitelist_pair(
            &owner,
            &second_pair_setup.first_token_id,
            &second_pair_setup.second_token_id,
            second_pair_setup.pair_wrapper.address_ref(),
        );
        router_setup.whitelist_pair(
            &owner,
            &third_pair_setup.first_token_id,
            &third_pair_setup.second_token_id,
            third_pair_setup.pair_wrapper.address_ref(),
        );

        // add initial liquidity
        first_pair_setup.add_liquidity(&owner, 1_000_000_000, 2_000_000_000);
        second_pair_setup.add_liquidity(&owner, 1_000_000_000, 6_000_000_000);
        third_pair_setup.add_liquidity(&owner, 1_000_000_000, 3_000_000_000);

        // setup price observations
        for _ in 1usize..=20 {
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
                            &managed_biguint!(1_000_000_000),
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
                            &managed_biguint!(1_000_000_000),
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
                            &managed_biguint!(1_000_000_000),
                        );
                    },
                )
                .assert_ok();
        }

        // setup dca module sc
        let dca_module_wrapper = b_mock.borrow_mut().create_sc_account(
            &rust_biguint!(0),
            Some(&owner),
            dca_module_builder,
            "dca module",
        );

        b_mock
            .borrow_mut()
            .execute_tx(&owner, &dca_module_wrapper, &rust_biguint!(0), |sc| {
                sc.init(
                    managed_address!(router_setup.router_wrapper.address_ref()),
                    1,
                );

                sc.paused_status().set(false);
            })
            .assert_ok();

        let pair_setups = vec![first_pair_setup, second_pair_setup, third_pair_setup];
        b_mock.borrow_mut().set_block_timestamp(START_TIME);

        DcaModuleSetup {
            b_mock,
            owner,
            user,
            current_time: START_TIME,
            pair_setups,
            router_setup,
            dca_module_wrapper,
        }
    }

    pub fn advance_time(&mut self, advance_by: Timestamp) {
        self.current_time += advance_by;
        self.b_mock
            .borrow_mut()
            .set_block_timestamp(self.current_time);
    }

    pub fn user_deposit(&self, tokens: &[TxTokenTransfer]) -> TxResult {
        self.b_mock.borrow_mut().execute_esdt_multi_transfer(
            &self.user,
            &self.dca_module_wrapper,
            tokens,
            |sc| {
                sc.deposit();
            },
        )
    }

    pub fn user_withdraw_part(&self, tokens: &[TxTokenTransfer]) -> TxResult {
        self.b_mock.borrow_mut().execute_tx(
            &self.user,
            &self.dca_module_wrapper,
            &rust_biguint!(0),
            |sc| {
                let mut esdt = MultiValueEncoded::new();
                for token in tokens {
                    esdt.push(
                        (
                            managed_token_id!(token.token_identifier.clone()),
                            token.nonce,
                            BigUint::from_bytes_be(&token.value.to_bytes_be()),
                        )
                            .into(),
                    );
                }

                sc.withdraw(esdt);
            },
        )
    }

    pub fn user_withdraw_all(&self) -> TxResult {
        self.b_mock.borrow_mut().execute_tx(
            &self.user,
            &self.dca_module_wrapper,
            &rust_biguint!(0),
            |sc| {
                sc.withdraw_all();
            },
        )
    }

    pub fn register_action(
        &self,
        trade_frequency: TradeFrequency,
        total_actions: TotalActions,
        input_token_id: &[u8],
        input_tokens_amount: u64,
        output_token_id: &[u8],
    ) -> TxResult {
        self.b_mock.borrow_mut().execute_tx(
            &self.user,
            &self.dca_module_wrapper,
            &rust_biguint!(0),
            |sc| {
                sc.register_action(
                    trade_frequency,
                    total_actions,
                    managed_token_id!(input_token_id),
                    managed_biguint!(input_tokens_amount),
                    managed_token_id!(output_token_id),
                );
            },
        )
    }

    pub fn execute_action(&self, action_id: ActionId, swap_args: Vec<DummySwapArgs>) -> TxResult {
        self.b_mock.borrow_mut().execute_tx(
            &self.owner,
            &self.dca_module_wrapper,
            &rust_biguint!(0),
            |sc| {
                let mut managed_swap_args = MultiValueEncoded::new();
                for swap_arg in swap_args {
                    let managed_arg = (
                        managed_address!(&swap_arg.pair_addr),
                        managed_token_id!(swap_arg.requested_token),
                        managed_biguint!(swap_arg.min_amount_out),
                    )
                        .into();
                    managed_swap_args.push(managed_arg);
                }

                sc.execute_action(action_id, managed_swap_args);
            },
        )
    }
}
