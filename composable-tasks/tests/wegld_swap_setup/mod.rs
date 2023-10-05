#![allow(deprecated)]

use std::cell::RefCell;
use std::rc::Rc;

use multiversx_sc::types::{Address, EsdtLocalRole};
use multiversx_sc_scenario::{
    managed_token_id, rust_biguint,
    testing_framework::{BlockchainStateWrapper, ContractObjWrapper},
    DebugApi,
};
use multiversx_wegld_swap_sc::EgldEsdtSwap;

pub static WEGLD_TOKEN_ID: &[u8] = b"WEGLD-123456";
pub static EGLD_TOKEN_ID: &[u8] = b"EGLD";

pub struct WegldSwapSetup<WegldSwapObjBuilder>
where
    WegldSwapObjBuilder: 'static + Copy + Fn() -> multiversx_wegld_swap_sc::ContractObj<DebugApi>,
{
    pub b_mock: Rc<RefCell<BlockchainStateWrapper>>,
    pub wegld_swap_wrapper:
        ContractObjWrapper<multiversx_wegld_swap_sc::ContractObj<DebugApi>, WegldSwapObjBuilder>,
}

impl<WegldSwapObjBuilder> WegldSwapSetup<WegldSwapObjBuilder>
where
    WegldSwapObjBuilder: 'static + Copy + Fn() -> multiversx_wegld_swap_sc::ContractObj<DebugApi>,
{
    pub fn new(
        b_mock: Rc<RefCell<BlockchainStateWrapper>>,
        wegld_swap_builder: WegldSwapObjBuilder,
        owner: &Address,
    ) -> Self {
        let rust_zero = rust_biguint!(0u64);
        let wegld_swap_wrapper = b_mock.borrow_mut().create_sc_account(
            &rust_zero,
            Some(owner),
            wegld_swap_builder,
            "wegld swap",
        );

        b_mock
            .borrow_mut()
            .execute_tx(owner, &wegld_swap_wrapper, &rust_zero, |sc| {
                sc.init(managed_token_id!(WEGLD_TOKEN_ID));
            })
            .assert_ok();

        let initial_token_balance = 10_000_000_000u64;
        b_mock.borrow_mut().set_esdt_balance(
            wegld_swap_wrapper.address_ref(),
            WEGLD_TOKEN_ID,
            &rust_biguint!(initial_token_balance),
        );
        b_mock.borrow_mut().set_egld_balance(
            wegld_swap_wrapper.address_ref(),
            &rust_biguint!(initial_token_balance),
        );

        let wegld_token_roles = [EsdtLocalRole::Mint, EsdtLocalRole::Burn];
        b_mock.borrow_mut().set_esdt_local_roles(
            wegld_swap_wrapper.address_ref(),
            WEGLD_TOKEN_ID,
            &wegld_token_roles[..],
        );


        WegldSwapSetup {
            b_mock,
            wegld_swap_wrapper,
        }
    }
}
