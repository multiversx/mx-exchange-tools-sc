use energy_dao::external_sc_interactions::{
    energy_dao_config::EnergyDAOConfigModule, farm_interactions::FarmInteractionsModule,
    locked_token_interactions::LockedTokenInteractionsModule,
    metastaking_interactions::MetastakingInteractionsModule,
};
use farm_with_locked_rewards::Farm;
use multiversx_sc::{
    codec::multi_types::MultiValue3,
    types::{Address, EsdtTokenPayment, MultiValueEncoded},
};
use multiversx_sc_scenario::{
    managed_address, managed_biguint, managed_token_id, rust_biguint,
    testing_framework::TxTokenTransfer, DebugApi,
};
use pair::Pair;

use crate::contract_setup::{EnergyDAOContractSetup, BASE_ASSET_TOKEN_ID, LOCKED_TOKEN_ID};

impl<
        EnergyDAOContractObjBuilder,
        EnergyFactoryObjBuilder,
        FeesCollectorObjBuilder,
        LockedTokenWrapperObjBuilder,
        PairObjBuilder,
        FarmObjBuilder,
        FarmStakingObjBuilder,
        FarmStakingProxyObjBuilder,
    >
    EnergyDAOContractSetup<
        EnergyDAOContractObjBuilder,
        EnergyFactoryObjBuilder,
        FeesCollectorObjBuilder,
        LockedTokenWrapperObjBuilder,
        PairObjBuilder,
        FarmObjBuilder,
        FarmStakingObjBuilder,
        FarmStakingProxyObjBuilder,
    >
where
    EnergyDAOContractObjBuilder: 'static + Copy + Fn() -> energy_dao::ContractObj<DebugApi>,
    EnergyFactoryObjBuilder: 'static + Copy + Fn() -> energy_factory::ContractObj<DebugApi>,
    FeesCollectorObjBuilder: 'static + Copy + Fn() -> fees_collector::ContractObj<DebugApi>,
    LockedTokenWrapperObjBuilder:
        'static + Copy + Fn() -> locked_token_wrapper::ContractObj<DebugApi>,
    PairObjBuilder: 'static + Copy + Fn() -> pair::ContractObj<DebugApi>,
    FarmObjBuilder: 'static + Copy + Fn() -> farm_with_locked_rewards::ContractObj<DebugApi>,
    FarmStakingObjBuilder: 'static + Copy + Fn() -> farm_staking::ContractObj<DebugApi>,
    FarmStakingProxyObjBuilder: 'static + Copy + Fn() -> farm_staking_proxy::ContractObj<DebugApi>,
{
    #[allow(dead_code)]
    pub fn add_farm(&mut self, farm_address: &Address) {
        self.b_mock
            .execute_tx(
                &self.owner_address,
                &self.energy_dao_wrapper,
                &rust_biguint!(0u64),
                |sc| {
                    let mut farms = MultiValueEncoded::new();
                    farms.push(managed_address!(farm_address));
                    sc.add_farms(farms);
                },
            )
            .assert_ok();
    }

    #[allow(dead_code)]
    pub fn add_metastaking_address(&mut self, metastaking_address: &Address) {
        self.b_mock
            .execute_tx(
                &self.owner_address,
                &self.energy_dao_wrapper,
                &rust_biguint!(0u64),
                |sc| {
                    let mut metastaking_addresses = MultiValueEncoded::new();
                    metastaking_addresses.push(managed_address!(metastaking_address));
                    sc.add_metastaking_addresses(metastaking_addresses);
                },
            )
            .assert_ok();
    }

    #[allow(dead_code)]
    pub fn call_pair_add_liquidity(
        &mut self,
        caller: &Address,
        first_token_id: &[u8],
        first_token_amount: u64,
        second_token_id: &[u8],
        second_token_amount: u64,
    ) -> u64 {
        let mut new_lp_amount = 0u64;
        let payments = vec![
            TxTokenTransfer {
                token_identifier: first_token_id.to_vec(),
                nonce: 0,
                value: rust_biguint!(first_token_amount),
            },
            TxTokenTransfer {
                token_identifier: second_token_id.to_vec(),
                nonce: 0,
                value: rust_biguint!(second_token_amount),
            },
        ];

        self.b_mock
            .execute_esdt_multi_transfer(caller, &self.pair_wrapper, &payments, |sc| {
                let MultiValue3 { 0: payments } =
                    sc.add_liquidity(managed_biguint!(1u64), managed_biguint!(1u64));
                new_lp_amount = payments.0.amount.to_u64().unwrap();
            })
            .assert_ok();

        new_lp_amount
    }

    #[allow(dead_code)]
    pub fn call_pair_remove_liquidity(
        &mut self,
        caller: &Address,
        payment_token: &[u8],
        payment_amount: u64,
    ) {
        self.b_mock
            .execute_esdt_transfer(
                caller,
                &self.pair_wrapper,
                payment_token,
                0,
                &rust_biguint!(payment_amount),
                |sc| {
                    sc.remove_liquidity(managed_biguint!(1), managed_biguint!(1));
                },
            )
            .assert_ok();
    }

    #[allow(dead_code)]
    pub fn enter_energy_dao_farm_endpoint(
        &mut self,
        sc_address: &Address,
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
                    sc.enter_farm_endpoint(managed_address!(sc_address));
                },
            )
            .assert_ok();
    }

    #[allow(dead_code)]
    pub fn enter_original_farm_endpoint(
        &mut self,
        caller: &Address,
        payment_token: &[u8],
        payment_amount: u64,
    ) {
        self.b_mock
            .execute_esdt_transfer(
                caller,
                &self.farm_wrapper,
                payment_token,
                0,
                &rust_biguint!(payment_amount),
                |sc| {
                    sc.enter_farm_endpoint(multiversx_sc::codec::multi_types::OptionalValue::None);
                },
            )
            .assert_ok();
    }

    #[allow(dead_code)]
    pub fn enter_energy_dao_metastaking_endpoint(
        &mut self,
        sc_address: &Address,
        caller: &Address,
        payment_token: &[u8],
        payment_amount: u64,
    ) -> u64 {
        let mut dual_yield_token_amount = 0u64;
        self.b_mock
            .execute_esdt_transfer(
                caller,
                &self.energy_dao_wrapper,
                payment_token,
                0u64,
                &rust_biguint!(payment_amount),
                |sc| {
                    let dual_yield_token =
                        sc.enter_metastaking_endpoint(managed_address!(sc_address));
                    dual_yield_token_amount = dual_yield_token.amount.to_u64().unwrap();
                },
            )
            .assert_ok();

        dual_yield_token_amount
    }

    #[allow(dead_code)]
    pub fn claim_user_rewards(
        &mut self,
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
                    sc.claim_user_rewards();
                },
            )
            .assert_ok();
    }

    #[allow(dead_code)]
    pub fn unstake_farm(
        &mut self,
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
                    sc.unstake_farm();
                },
            )
            .assert_ok();
    }

    #[allow(dead_code)]
    pub fn unbond_farm(
        &mut self,
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
                    sc.unbond_farm();
                },
            )
            .assert_ok();
    }

    #[allow(dead_code)]
    pub fn claim_user_metastaking_rewards(
        &mut self,
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
                    sc.claim_metastaking_rewards();
                },
            )
            .assert_ok();
    }

    #[allow(dead_code)]
    pub fn unstake_metastaking(
        &mut self,
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
                    sc.unstake_metastaking();
                },
            )
            .assert_ok();
    }

    #[allow(dead_code)]
    pub fn unbond_metastaking(
        &mut self,
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
                    sc.unbond_metastaking_endpoint();
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
}
