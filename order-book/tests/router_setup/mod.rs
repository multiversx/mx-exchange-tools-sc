#![allow(deprecated)]

use std::cell::RefCell;
use std::rc::Rc;

use multiversx_sc::{codec::multi_types::OptionalValue, types::Address};
use multiversx_sc_scenario::{
    managed_address, managed_token_id, rust_biguint,
    testing_framework::{BlockchainStateWrapper, ContractObjWrapper},
    DebugApi,
};

use router::{config::ConfigModule, factory::PairTokens, *};

pub struct RouterSetup<RouterObjBuilder>
where
    RouterObjBuilder: 'static + Copy + Fn() -> router::ContractObj<DebugApi>,
{
    pub b_mock: Rc<RefCell<BlockchainStateWrapper>>,
    pub router_wrapper: ContractObjWrapper<router::ContractObj<DebugApi>, RouterObjBuilder>,
}

impl<RouterObjBuilder> RouterSetup<RouterObjBuilder>
where
    RouterObjBuilder: 'static + Copy + Fn() -> router::ContractObj<DebugApi>,
{
    pub fn new(
        b_mock: Rc<RefCell<BlockchainStateWrapper>>,
        router_builder: RouterObjBuilder,
        owner: &Address,
        template_address: &Address,
    ) -> Self {
        let rust_zero = rust_biguint!(0u64);
        let router_wrapper = b_mock.borrow_mut().create_sc_account(
            &rust_zero,
            Some(owner),
            router_builder,
            "router",
        );

        b_mock
            .borrow_mut()
            .execute_tx(owner, &router_wrapper, &rust_zero, |sc| {
                sc.init(OptionalValue::Some(managed_address!(template_address)));
            })
            .assert_ok();

        RouterSetup {
            b_mock,
            router_wrapper,
        }
    }

    // Instead of actually deploying tthe pairs through the router
    // We simply whitelist them in the router contract
    pub fn whitelist_pair(
        &mut self,
        caller: &Address,
        first_token_id: &[u8],
        second_token_id: &[u8],
        pair_address: &Address,
    ) {
        self.b_mock
            .borrow_mut()
            .execute_tx(caller, &self.router_wrapper, &rust_biguint!(0u64), |sc| {
                sc.pair_map().insert(
                    PairTokens {
                        first_token_id: managed_token_id!(first_token_id),
                        second_token_id: managed_token_id!(second_token_id),
                    },
                    managed_address!(pair_address),
                );
            })
            .assert_ok();
    }
}
