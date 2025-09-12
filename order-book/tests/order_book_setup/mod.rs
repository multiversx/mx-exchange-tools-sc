#![allow(deprecated)]

use std::{cell::RefCell, rc::Rc};

use crate::pair_setup::PairSetup;
use crate::router_setup::RouterSetup;

use multiversx_sc::types::{Address, ManagedVec, MultiValueEncoded};
use multiversx_sc_scenario::{
    imports::{BlockchainStateWrapper, TxResult},
    managed_address, managed_biguint, managed_token_id, rust_biguint,
    testing_framework::ContractObjWrapper,
    DebugApi,
};

use order_book::{
    actors::{
        executor::{ExecutorModule, RouterEndpointName, SwapOperationType, SwapStatus},
        maker::MakerModule,
    },
    pause::PauseModule,
    storage::{
        common_storage::{CommonStorageModule, Percent},
        order::{OrderDuration, OrderId},
    },
    OrderBook,
};
use pair::safe_price::SafePriceModule;

pub static TOKEN_IDS: &[&[u8]] = &[b"FIRST-123456", b"SECOND-123456", b"THIRD-123456"];
pub static LP_TOKEN_IDS: &[&[u8]] = &[b"LPFIRST-123456", b"LPSECOND-123456", b"LPTHIRD-123456"];
pub static WEGLD_TOKEN_ID: &[u8] = b"WEGLD-123456";
pub static DUAL_YIELD_TOKEN_ID: &[u8] = b"DUALYIELD-123456";

pub const USER_BALANCE: u64 = 100_000;

pub struct UnmanagedSwapOperationType {
    pub pair_address: Address,
    pub endpoint_name: RouterEndpointName,
    pub output_token_id: Vec<u8>,
}

pub struct ExecuteOrdersArg {
    pub order_id: OrderId,
    pub amount_to_swap: u64,
    pub swap_args: Vec<UnmanagedSwapOperationType>,
}

pub struct OrderBookSetup<PairBuilder, RouterBuilder, OrderBookBuilder>
where
    PairBuilder: 'static + Copy + Fn() -> pair::ContractObj<DebugApi>,
    RouterBuilder: 'static + Copy + Fn() -> router::ContractObj<DebugApi>,
    OrderBookBuilder: 'static + Copy + Fn() -> order_book::ContractObj<DebugApi>,
{
    pub b_mock: Rc<RefCell<BlockchainStateWrapper>>,
    pub owner: Address,
    pub user: Address,
    pub treasury: Address,
    pub pair_setups: Vec<PairSetup<PairBuilder>>,
    pub router_setup: RouterSetup<RouterBuilder>,
    pub order_book_wrapper: ContractObjWrapper<order_book::ContractObj<DebugApi>, OrderBookBuilder>,
}

impl<PairBuilder, RouterBuilder, OrderBookBuilder>
    OrderBookSetup<PairBuilder, RouterBuilder, OrderBookBuilder>
where
    PairBuilder: 'static + Copy + Fn() -> pair::ContractObj<DebugApi>,
    RouterBuilder: 'static + Copy + Fn() -> router::ContractObj<DebugApi>,
    OrderBookBuilder: 'static + Copy + Fn() -> order_book::ContractObj<DebugApi>,
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
        order_book_builder: OrderBookBuilder,
    ) -> Self {
        let rust_zero = rust_biguint!(0);

        let b_mock_simple = BlockchainStateWrapper::new();
        let b_mock_ref = RefCell::new(b_mock_simple);
        let b_mock = Rc::new(b_mock_ref);

        let owner = b_mock.borrow_mut().create_user_account(&rust_zero);
        let user = b_mock.borrow_mut().create_user_account(&rust_zero);
        let treasury = b_mock.borrow_mut().create_user_account(&rust_zero);
        let mut first_pair_setup = PairSetup::new(
            b_mock.clone(),
            pair_builder,
            &owner,
            None,
            TOKEN_IDS[0],
            TOKEN_IDS[1],
            LP_TOKEN_IDS[0],
        );
        let mut second_pair_setup = PairSetup::new(
            b_mock.clone(),
            pair_builder,
            &owner,
            None,
            TOKEN_IDS[0],
            TOKEN_IDS[2],
            LP_TOKEN_IDS[1],
        );
        let mut third_pair_setup = PairSetup::new(
            b_mock.clone(),
            pair_builder,
            &owner,
            None,
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

        b_mock
            .borrow_mut()
            .set_esdt_balance(&user, TOKEN_IDS[0], &rust_biguint!(USER_BALANCE));
        b_mock
            .borrow_mut()
            .set_esdt_balance(&user, TOKEN_IDS[1], &rust_biguint!(USER_BALANCE));
        b_mock
            .borrow_mut()
            .set_esdt_balance(&user, TOKEN_IDS[2], &rust_biguint!(USER_BALANCE));

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

        // setup order book sc
        let order_book_wrapper = b_mock.borrow_mut().create_sc_account(
            &rust_biguint!(0),
            Some(&owner),
            order_book_builder,
            "order book",
        );

        b_mock
            .borrow_mut()
            .execute_tx(&owner, &order_book_wrapper, &rust_biguint!(0), |sc| {
                let mut admins = MultiValueEncoded::new();
                admins.push(managed_address!(&owner));

                sc.init(
                    managed_address!(router_setup.router_wrapper.address_ref()),
                    managed_address!(&treasury),
                    1_000, // 10%
                    2_000, // 20%
                    admins,
                );

                sc.paused_status().set(false);

                let _ = sc.executor_whitelist().insert(managed_address!(&owner));
            })
            .assert_ok();

        let pair_setups = vec![first_pair_setup, second_pair_setup, third_pair_setup];

        OrderBookSetup {
            b_mock,
            owner,
            user,
            treasury,
            pair_setups,
            router_setup,
            order_book_wrapper,
        }
    }

    pub fn call_create_order(
        &self,
        payment_token: &[u8],
        payment_amount: u64,
        output_token: &[u8],
        min_total_output: u64,
        duration: OrderDuration,
        opt_executor_fee: Option<Percent>,
    ) -> (TxResult, OrderId) {
        let mut order_id = 0;
        let tx_result = self.b_mock.borrow_mut().execute_esdt_transfer(
            &self.user,
            &self.order_book_wrapper,
            payment_token,
            0,
            &rust_biguint!(payment_amount),
            |sc| {
                order_id = sc.create_order(
                    managed_token_id!(output_token),
                    managed_biguint!(min_total_output),
                    duration,
                    opt_executor_fee.into(),
                );
            },
        );

        (tx_result, order_id)
    }

    pub fn call_cancel_order(&self, order_id: OrderId) -> TxResult {
        self.b_mock.borrow_mut().execute_tx(
            &self.user,
            &self.order_book_wrapper,
            &rust_biguint!(0),
            |sc| {
                sc.cancel_order(order_id);
            },
        )
    }

    pub fn call_execute_orders(&self, args: &[ExecuteOrdersArg]) -> Vec<SwapStatus> {
        let mut return_value = Vec::new();

        self.b_mock
            .borrow_mut()
            .execute_tx(
                &self.owner,
                &self.order_book_wrapper,
                &rust_biguint!(0),
                |sc| {
                    let mut managed_args = MultiValueEncoded::new();
                    for arg in args {
                        let mut managed_swap_args = ManagedVec::new();
                        for swap_arg in &arg.swap_args {
                            let managed_swap_arg = SwapOperationType {
                                pair_address: managed_address!(&swap_arg.pair_address),
                                output_token_id: managed_token_id!(swap_arg
                                    .output_token_id
                                    .clone()),
                                endpoint_name: swap_arg.endpoint_name,
                            };

                            managed_swap_args.push(managed_swap_arg);
                        }

                        managed_args.push(
                            (arg.order_id, arg.amount_to_swap.into(), managed_swap_args).into(),
                        );
                    }

                    let managed_return = sc.execute_orders(managed_args);
                    for value in managed_return {
                        return_value.push(value);
                    }
                },
            )
            .assert_ok();

        return_value
    }
}
