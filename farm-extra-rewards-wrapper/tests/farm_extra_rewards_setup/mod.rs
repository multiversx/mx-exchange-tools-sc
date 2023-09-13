#![allow(deprecated)]

use std::{cell::RefCell, rc::Rc};

use auto_farm::whitelists::farms_whitelist::FarmsWhitelistModule;
use farm_extra_rewards_wrapper::FarmExtraRewardsWrapper;
use multiversx_sc::types::{Address, ManagedAddress, MultiValueEncoded};
use multiversx_sc_scenario::{
    managed_biguint, rust_biguint,
    testing_framework::{BlockchainStateWrapper, ContractObjWrapper},
    DebugApi,
};
use tests_common::farm_staking_setup::DIVISION_SAFETY_CONSTANT;

pub struct ExtraRewSetup<ScBuilder>
where
    ScBuilder: 'static + Copy + Fn() -> farm_extra_rewards_wrapper::ContractObj<DebugApi>,
{
    pub b_mock: Rc<RefCell<BlockchainStateWrapper>>,
    pub owner: Address,
    pub sc_wrapper:
        ContractObjWrapper<farm_extra_rewards_wrapper::ContractObj<DebugApi>, ScBuilder>,
}

impl<ScBuilder> ExtraRewSetup<ScBuilder>
where
    ScBuilder: 'static + Copy + Fn() -> farm_extra_rewards_wrapper::ContractObj<DebugApi>,
{
    pub fn new(
        b_mock: Rc<RefCell<BlockchainStateWrapper>>,
        owner: Address,
        builder: ScBuilder,
    ) -> Self {
        let sc_wrapper = b_mock.borrow_mut().create_sc_account(
            &rust_biguint!(0),
            Some(&owner),
            builder,
            "Farm extra rewards",
        );

        b_mock
            .borrow_mut()
            .execute_tx(&owner, &sc_wrapper, &rust_biguint!(0), |sc| {
                sc.init(managed_biguint!(DIVISION_SAFETY_CONSTANT));
            })
            .assert_ok();

        Self {
            b_mock,
            owner,
            sc_wrapper,
        }
    }

    pub fn add_farms(&mut self, farms: Vec<Address>) {
        self.b_mock
            .borrow_mut()
            .execute_tx(&self.owner, &self.sc_wrapper, &rust_biguint!(0), |sc| {
                let mut args = MultiValueEncoded::new();
                for farm in farms {
                    args.push(ManagedAddress::from_address(&farm));
                }

                sc.add_farms(args);
            })
            .assert_ok();
    }
}
