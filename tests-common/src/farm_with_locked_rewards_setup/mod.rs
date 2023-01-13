#![cfg(feature = "enable-tests-common")]

use std::{cell::RefCell, rc::Rc};

use config::ConfigModule;
use elrond_wasm::{
    elrond_codec::multi_types::OptionalValue,
    storage::mappers::StorageTokenWrapper,
    types::{Address, BigInt, EsdtLocalRole, MultiValueEncoded},
};
use elrond_wasm_debug::{
    managed_address, managed_biguint, managed_token_id, rust_biguint,
    testing_framework::{BlockchainStateWrapper, ContractObjWrapper},
    DebugApi,
};

mod fees_collector_mock;
use fees_collector_mock::*;

use elrond_wasm_modules::pause::PauseModule;
use energy_factory::{energy::EnergyModule, SimpleLockEnergy};
use energy_query::{Energy, EnergyQueryModule};
use farm_boosted_yields::boosted_yields_factors::BoostedYieldsFactorsModule;
use farm_boosted_yields::FarmBoostedYieldsModule;
use farm_token::FarmTokenModule;
use farm_with_locked_rewards::Farm;
use locking_module::lock_with_energy_module::LockWithEnergyModule;
use pausable::{PausableModule, State};
use sc_whitelist_module::SCWhitelistModule;
use simple_lock::locked_token::LockedTokenModule;

pub static REWARD_TOKEN_ID: &[u8] = b"MEX-123456";
pub static LOCKED_REWARD_TOKEN_ID: &[u8] = b"LOCKED-123456";
pub static LEGACY_LOCKED_TOKEN_ID: &[u8] = b"LEGACY-123456";
pub static FARMING_TOKEN_ID: &[&[u8]] = &[b"LPTOK-123456", b"LPTOK-654321"];
pub static FARM_TOKEN_ID: &[&[u8]] = &[b"FIRFARM-123456", b"SECFARM-123456"];
const DIV_SAFETY: u64 = 1_000_000_000_000;
const PER_BLOCK_REWARD_AMOUNT: u64 = 1_000;
const FARMING_TOKEN_BALANCE: u64 = 1_000_000_000;

pub const BOOSTED_YIELDS_PERCENTAGE: u64 = 2_500; // 25%
pub const USER_REWARDS_BASE_CONST: u64 = 10;
pub const USER_REWARDS_ENERGY_CONST: u64 = 1;
pub const USER_REWARDS_FARM_CONST: u64 = 0;
pub const MIN_ENERGY_AMOUNT_FOR_BOOSTED_YIELDS: u64 = 1;
pub const MIN_FARM_AMOUNT_FOR_BOOSTED_YIELDS: u64 = 1;

pub const EPOCHS_IN_YEAR: u64 = 365;

pub static LOCK_OPTIONS: &[u64] = &[EPOCHS_IN_YEAR, 2 * EPOCHS_IN_YEAR, 4 * EPOCHS_IN_YEAR];
pub static PENALTY_PERCENTAGES: &[u64] = &[4_000, 6_000, 8_000];

pub struct FarmSetup<FarmObjBuilder, EnergyFactoryBuilder>
where
    FarmObjBuilder: 'static + Copy + Fn() -> farm_with_locked_rewards::ContractObj<DebugApi>,
    EnergyFactoryBuilder: 'static + Copy + Fn() -> energy_factory::ContractObj<DebugApi>,
{
    pub b_mock: Rc<RefCell<BlockchainStateWrapper>>,
    pub owner: Address,
    pub first_user: Address,
    pub second_user: Address,
    pub third_user: Address,
    pub farm_wrappers:
        Vec<ContractObjWrapper<farm_with_locked_rewards::ContractObj<DebugApi>, FarmObjBuilder>>,
    pub energy_factory_wrapper:
        ContractObjWrapper<energy_factory::ContractObj<DebugApi>, EnergyFactoryBuilder>,
}

impl<FarmObjBuilder, EnergyFactoryBuilder> FarmSetup<FarmObjBuilder, EnergyFactoryBuilder>
where
    FarmObjBuilder: 'static + Copy + Fn() -> farm_with_locked_rewards::ContractObj<DebugApi>,
    EnergyFactoryBuilder: 'static + Copy + Fn() -> energy_factory::ContractObj<DebugApi>,
{
    pub fn new(farm_builder: FarmObjBuilder, energy_factory_builder: EnergyFactoryBuilder) -> Self {
        let rust_zero = rust_biguint!(0);
        let mut b_mock = BlockchainStateWrapper::new();
        let owner = b_mock.create_user_account(&rust_zero);

        // needed for metabonding signatures

        // address:user1 from mandos
        let first_user = Address::from(hex_literal::hex!(
            "75736572315F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F"
        ));
        b_mock.create_user_account_fixed_address(&first_user, &rust_zero);

        // address:user2 from mandos
        let second_user = Address::from(hex_literal::hex!(
            "75736572325F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F"
        ));
        b_mock.create_user_account_fixed_address(&second_user, &rust_zero);

        let third_user = b_mock.create_user_account(&rust_zero);
        let first_farm_wrapper = b_mock.create_sc_account(
            &rust_zero,
            Some(&owner),
            farm_builder,
            "farm-with-locked-rewards.wasm",
        );
        let second_farm_wrapper = b_mock.create_sc_account(
            &rust_zero,
            Some(&owner),
            farm_builder,
            "farm-with-locked-rewards.wasm",
        );
        let energy_factory_wrapper = b_mock.create_sc_account(
            &rust_zero,
            Some(&owner),
            energy_factory_builder,
            "energy_factory.wasm",
        );
        let fees_collector_mock = b_mock.create_sc_account(
            &rust_zero,
            Some(&owner),
            FeesCollectorMock::new,
            "fees collector mock",
        );

        b_mock
            .execute_tx(&owner, &energy_factory_wrapper, &rust_zero, |sc| {
                let mut lock_options = MultiValueEncoded::new();
                for (option, penalty) in LOCK_OPTIONS.iter().zip(PENALTY_PERCENTAGES.iter()) {
                    lock_options.push((*option, *penalty).into());
                }

                sc.init(
                    managed_token_id!(REWARD_TOKEN_ID),
                    managed_token_id!(LEGACY_LOCKED_TOKEN_ID),
                    managed_address!(fees_collector_mock.address_ref()),
                    0,
                    lock_options,
                );

                sc.locked_token()
                    .set_token_id(managed_token_id!(LOCKED_REWARD_TOKEN_ID));
                sc.set_paused(false);
            })
            .assert_ok();

        let farm_wrappers = vec![first_farm_wrapper, second_farm_wrapper];
        for (i, farm_wrapper) in farm_wrappers.iter().enumerate() {
            b_mock
                .execute_tx(&owner, farm_wrapper, &rust_zero, |sc| {
                    let reward_token_id = managed_token_id!(REWARD_TOKEN_ID);
                    let farming_token_id = managed_token_id!(FARMING_TOKEN_ID[i]);
                    let division_safety_constant = managed_biguint!(DIV_SAFETY);
                    let pair_address = managed_address!(&Address::zero());

                    sc.init(
                        reward_token_id,
                        farming_token_id,
                        division_safety_constant,
                        pair_address,
                        managed_address!(&owner),
                        MultiValueEncoded::new(),
                    );

                    let farm_token_id = managed_token_id!(FARM_TOKEN_ID[i]);
                    sc.farm_token().set_token_id(farm_token_id);
                    sc.set_locking_sc_address(managed_address!(
                        energy_factory_wrapper.address_ref()
                    ));
                    sc.set_lock_epochs(*LOCK_OPTIONS.last().unwrap());

                    sc.add_sc_address_to_whitelist(managed_address!(&first_user));
                    sc.add_sc_address_to_whitelist(managed_address!(&second_user));
                    sc.add_sc_address_to_whitelist(managed_address!(&third_user));

                    sc.per_block_reward_amount()
                        .set(&managed_biguint!(PER_BLOCK_REWARD_AMOUNT));

                    sc.state().set(State::Active);
                    sc.produce_rewards_enabled().set(true);
                    sc.set_energy_factory_address(managed_address!(
                        energy_factory_wrapper.address_ref()
                    ));

                    sc.set_boosted_yields_factors(
                        managed_biguint!(USER_REWARDS_BASE_CONST),
                        managed_biguint!(USER_REWARDS_ENERGY_CONST),
                        managed_biguint!(USER_REWARDS_FARM_CONST),
                        managed_biguint!(MIN_ENERGY_AMOUNT_FOR_BOOSTED_YIELDS),
                        managed_biguint!(MIN_FARM_AMOUNT_FOR_BOOSTED_YIELDS),
                    );
                    sc.set_boosted_yields_rewards_percentage(BOOSTED_YIELDS_PERCENTAGE);
                })
                .assert_ok();

            let farm_token_roles = [
                EsdtLocalRole::NftCreate,
                EsdtLocalRole::NftAddQuantity,
                EsdtLocalRole::NftBurn,
            ];
            b_mock.set_esdt_local_roles(
                farm_wrapper.address_ref(),
                FARM_TOKEN_ID[i],
                &farm_token_roles[..],
            );

            let farming_token_roles = [EsdtLocalRole::Burn];
            b_mock.set_esdt_local_roles(
                farm_wrapper.address_ref(),
                FARMING_TOKEN_ID[i],
                &farming_token_roles[..],
            );

            b_mock.set_esdt_balance(
                &first_user,
                FARMING_TOKEN_ID[i],
                &rust_biguint!(FARMING_TOKEN_BALANCE),
            );
            b_mock.set_esdt_balance(
                &second_user,
                FARMING_TOKEN_ID[i],
                &rust_biguint!(FARMING_TOKEN_BALANCE),
            );
            b_mock.set_esdt_balance(
                &third_user,
                FARMING_TOKEN_ID[i],
                &rust_biguint!(FARMING_TOKEN_BALANCE),
            );
        }

        let locked_reward_token_roles = [
            EsdtLocalRole::NftCreate,
            EsdtLocalRole::NftAddQuantity,
            EsdtLocalRole::NftBurn,
            EsdtLocalRole::Transfer,
        ];
        b_mock.set_esdt_local_roles(
            energy_factory_wrapper.address_ref(),
            LOCKED_REWARD_TOKEN_ID,
            &locked_reward_token_roles[..],
        );

        b_mock
            .execute_tx(&owner, &energy_factory_wrapper, &rust_zero, |sc| {
                for farm_wrapper in &farm_wrappers {
                    sc.sc_whitelist_addresses()
                        .add(&managed_address!(farm_wrapper.address_ref()));
                }
            })
            .assert_ok();

        let b_mock_ref = RefCell::new(b_mock);
        let b_mock_rc = Rc::new(b_mock_ref);

        FarmSetup {
            b_mock: b_mock_rc,
            owner,
            first_user,
            second_user,
            third_user,
            farm_wrappers,
            energy_factory_wrapper,
        }
    }

    pub fn set_user_energy(
        &mut self,
        user: &Address,
        energy: u64,
        last_update_epoch: u64,
        locked_tokens: u64,
    ) {
        self.b_mock
            .borrow_mut()
            .execute_tx(
                &self.owner,
                &self.energy_factory_wrapper,
                &rust_biguint!(0),
                |sc| {
                    sc.user_energy(&managed_address!(user)).set(&Energy::new(
                        BigInt::from(managed_biguint!(energy)),
                        last_update_epoch,
                        managed_biguint!(locked_tokens),
                    ));
                },
            )
            .assert_ok();
    }

    pub fn enter_farm(&mut self, farm_index: usize, user: &Address, farming_token_amount: u64) {
        self.b_mock
            .borrow_mut()
            .execute_esdt_transfer(
                user,
                &self.farm_wrappers[farm_index],
                FARMING_TOKEN_ID[farm_index],
                0,
                &rust_biguint!(farming_token_amount),
                |sc| {
                    let enter_farm_result =
                        sc.enter_farm_endpoint(OptionalValue::Some(managed_address!(user)));
                    let (out_farm_token, _reward_token) = enter_farm_result.into_tuple();
                    assert_eq!(
                        out_farm_token.token_identifier,
                        managed_token_id!(FARM_TOKEN_ID[farm_index])
                    );
                    assert_eq!(
                        out_farm_token.amount,
                        managed_biguint!(farming_token_amount)
                    );
                },
            )
            .assert_ok();
    }

    pub fn claim_rewards(
        &mut self,
        farm_index: usize,
        user: &Address,
        farm_token_nonce: u64,
        farm_token_amount: u64,
    ) -> u64 {
        let mut result = 0;
        self.b_mock
            .borrow_mut()
            .execute_esdt_transfer(
                user,
                &self.farm_wrappers[farm_index],
                FARM_TOKEN_ID[farm_index],
                farm_token_nonce,
                &rust_biguint!(farm_token_amount),
                |sc| {
                    let (out_farm_token, out_reward_token) = sc
                        .claim_rewards_endpoint(OptionalValue::Some(managed_address!(user)))
                        .into_tuple();
                    assert_eq!(
                        out_farm_token.token_identifier,
                        managed_token_id!(FARM_TOKEN_ID[farm_index])
                    );
                    assert_eq!(out_farm_token.amount, managed_biguint!(farm_token_amount));

                    if out_reward_token.amount > 0 {
                        assert_eq!(
                            out_reward_token.token_identifier,
                            managed_token_id!(LOCKED_REWARD_TOKEN_ID)
                        );
                        assert_eq!(out_reward_token.token_nonce, 1);
                    } else {
                        assert_eq!(
                            out_reward_token.token_identifier,
                            managed_token_id!(REWARD_TOKEN_ID)
                        );
                        assert_eq!(out_reward_token.token_nonce, 0);
                    }

                    result = out_reward_token.amount.to_u64().unwrap();
                },
            )
            .assert_ok();

        result
    }

    pub fn exit_farm(
        &mut self,
        farm_index: usize,
        user: &Address,
        farm_token_nonce: u64,
        farm_token_amount: u64,
        exit_farm_amount: u64,
    ) {
        self.b_mock
            .borrow_mut()
            .execute_esdt_transfer(
                user,
                &self.farm_wrappers[farm_index],
                FARM_TOKEN_ID[farm_index],
                farm_token_nonce,
                &rust_biguint!(farm_token_amount),
                |sc| {
                    let _ = sc.exit_farm_endpoint(
                        managed_biguint!(exit_farm_amount),
                        OptionalValue::Some(managed_address!(user)),
                    );
                },
            )
            .assert_ok();
    }
}
