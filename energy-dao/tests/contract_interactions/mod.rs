use energy_dao::external_sc_interactions::{
    farm_config::FarmConfigModule, farm_interactions::FarmInteractionsModule,
    locked_token_interactions::LockedTokenInteractionsModule,
};
use multiversx_sc::{
    codec::multi_types::MultiValue3,
    types::{Address, EsdtTokenPayment, MultiValueEncoded},
};
use multiversx_sc_scenario::{
    managed_address, managed_biguint, managed_token_id, rust_biguint, DebugApi,
};

use crate::contract_setup::{
    EnergyDAOContractSetup, BASE_ASSET_TOKEN_ID, LOCKED_TOKEN_ID, SFT_ROLES,
};

impl<
        EnergyDAOContractObjBuilder,
        EnergyFactoryObjBuilder,
        FeesCollectorObjBuilder,
        LockedTokenWrapperObjBuilder,
        FarmObjBuilder,
    >
    EnergyDAOContractSetup<
        EnergyDAOContractObjBuilder,
        EnergyFactoryObjBuilder,
        FeesCollectorObjBuilder,
        LockedTokenWrapperObjBuilder,
        FarmObjBuilder,
    >
where
    EnergyDAOContractObjBuilder: 'static + Copy + Fn() -> energy_dao::ContractObj<DebugApi>,
    EnergyFactoryObjBuilder: 'static + Copy + Fn() -> energy_factory::ContractObj<DebugApi>,
    FeesCollectorObjBuilder: 'static + Copy + Fn() -> fees_collector::ContractObj<DebugApi>,
    LockedTokenWrapperObjBuilder:
        'static + Copy + Fn() -> locked_token_wrapper::ContractObj<DebugApi>,
    FarmObjBuilder: 'static + Copy + Fn() -> farm_with_locked_rewards::ContractObj<DebugApi>,
{
    pub fn add_farm(&mut self, farm_address: &Address, wrapped_token: &[u8], unstake_token: &[u8]) {
        self.b_mock.set_esdt_local_roles(
            self.energy_dao_wrapper.address_ref(),
            wrapped_token,
            SFT_ROLES,
        );
        self.b_mock.set_esdt_local_roles(
            self.energy_dao_wrapper.address_ref(),
            unstake_token,
            SFT_ROLES,
        );

        self.b_mock
            .execute_tx(
                &self.owner_address,
                &self.energy_dao_wrapper,
                &rust_biguint!(0u64),
                |sc| {
                    let mut farms = MultiValueEncoded::new();
                    let farm_data = MultiValue3::from((
                        managed_address!(farm_address),
                        managed_token_id!(wrapped_token),
                        managed_token_id!(unstake_token),
                    ));
                    farms.push(farm_data);
                    sc.add_farms(farms);
                },
            )
            .assert_ok();
    }

    pub fn enter_farm_endpoint(
        &mut self,
        farm_address: &Address,
        caller: &Address,
        payment_token: &[u8],
        payment_amount: u64,
    ) {
        self.b_mock
            .execute_esdt_transfer(
                caller,
                &self.energy_dao_wrapper,
                payment_token,
                0,
                &rust_biguint!(payment_amount),
                |sc| {
                    sc.enter_farm_endpoint(managed_address!(farm_address));
                },
            )
            .assert_ok();
    }

    pub fn claim_farm_rewards(&mut self, farm_address: &Address) {
        self.b_mock
            .execute_tx(
                &self.owner_address,
                &self.energy_dao_wrapper,
                &rust_biguint!(0u64),
                |sc| {
                    sc.claim_farm_rewards(managed_address!(farm_address));
                },
            )
            .assert_ok();
    }

    pub fn claim_user_rewards(
        &mut self,
        farm_address: &Address,
        caller: &Address,
        payment_token: &[u8],
        payment_nonce: u64,
        payment_amount: u64,
    ) {
        self.b_mock
            .execute_esdt_transfer(
                caller,
                &self.energy_dao_wrapper,
                payment_token,
                payment_nonce,
                &rust_biguint!(payment_amount),
                |sc| {
                    sc.claim_user_rewards(managed_address!(farm_address));
                },
            )
            .assert_ok();
    }

    pub fn unstake_farm(
        &mut self,
        farm_address: &Address,
        caller: &Address,
        payment_token: &[u8],
        payment_token_nonce: u64,
        payment_amount: u64,
    ) {
        self.b_mock
            .execute_esdt_transfer(
                caller,
                &self.energy_dao_wrapper,
                payment_token,
                payment_token_nonce,
                &rust_biguint!(payment_amount),
                |sc| {
                    sc.unstake_farm(managed_address!(farm_address));
                },
            )
            .assert_ok();
    }

    pub fn unbond_farm(
        &mut self,
        farm_address: &Address,
        caller: &Address,
        payment_token: &[u8],
        payment_token_nonce: u64,
        payment_amount: u64,
    ) {
        self.b_mock
            .execute_esdt_transfer(
                caller,
                &self.energy_dao_wrapper,
                payment_token,
                payment_token_nonce,
                &rust_biguint!(payment_amount),
                |sc| {
                    sc.unbond_farm(managed_address!(farm_address));
                },
            )
            .assert_ok();
    }

    pub fn lock_energy_tokens(&mut self, payment_amount: u64, lock_epoch: u64) {
        self.b_mock
            .execute_esdt_transfer(
                &self.owner_address,
                &self.energy_dao_wrapper,
                BASE_ASSET_TOKEN_ID,
                0,
                &rust_biguint!(payment_amount),
                |sc| {
                    let mut internal_locked_tokens = sc.internal_locked_tokens().get();
                    sc.lock_energy_tokens(lock_epoch);

                    let new_expected_payment = EsdtTokenPayment::new(
                        managed_token_id!(LOCKED_TOKEN_ID),
                        1,
                        managed_biguint!(payment_amount),
                    );
                    internal_locked_tokens.push(new_expected_payment);
                    assert_eq!(sc.internal_locked_tokens().get(), internal_locked_tokens);
                },
            )
            .assert_ok();
    }

    pub fn setup_new_user(&mut self, token_id: &[u8], token_amount: u64) -> Address {
        let rust_zero = rust_biguint!(0);

        let new_user = self.b_mock.create_user_account(&rust_zero);
        self.b_mock
            .set_esdt_balance(&new_user, token_id, &rust_biguint!(token_amount));
        new_user
    }

    pub fn check_user_balance(&self, address: &Address, token_id: &[u8], token_balance: u64) {
        self.b_mock
            .check_esdt_balance(address, token_id, &rust_biguint!(token_balance));
    }
}
