#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use multiversx_sc::storage::StorageKey;

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Copy, Clone, Debug)]
pub enum State {
    Inactive,
    Active,
    PartialActive,
}

pub static LP_TOKEN_ID_STORAGE_KEY: &[u8] = b"lpTokenIdentifier";
pub static FIRST_TOKEN_ID_STORAGE_KEY: &[u8] = b"first_token_id";
pub static SECOND_TOKEN_ID_STORAGE_KEY: &[u8] = b"second_token_id";
pub static PAIR_CONTRACT_ADDRESS_STORAGE_KEY: &[u8] = b"pair_contract_address";
pub static FARMING_TOKEN_ID_STORAGE_KEY: &[u8] = b"farming_token_id";
pub static FARM_TOKEN_ID_STORAGE_KEY: &[u8] = b"farm_token_id";
pub static LP_FARM_ADDRESS_STORAGE_KEY: &[u8] = b"lpFarmAddress";
pub static STAKING_FARM_ADDRESS_STORAGE_KEY: &[u8] = b"stakingFarmAddress";
pub static FARM_STATE_STORAGE_KEY: &[u8] = b"state";
pub static LP_FARM_TOKEN_ID_STORAGE_KEY: &[u8] = b"lpFarmTokenId";
pub static DUAL_YIELD_TOKEN_ID_STORAGE_KEY: &[u8] = b"dualYieldTokenId";
pub static STAKING_TOKEN_ID_STORAGE_KEY: &[u8] = b"stakingTokenId";
pub static DIVISION_SAFETY_CONSTANT_STORAGE_KEY: &[u8] = b"division_safety_constant";
pub static MINIMUM_FARMING_EPOCHS_STORAGE_KEY: &[u8] = b"minimum_farming_epochs";

#[multiversx_sc::module]
pub trait ReadExternalStorageModule {
    fn get_lp_token_id_mapper(
        &self,
        sc_address: ManagedAddress,
    ) -> SingleValueMapper<TokenIdentifier, ManagedAddress> {
        SingleValueMapper::<_, _, ManagedAddress>::new_from_address(
            sc_address,
            StorageKey::new(LP_TOKEN_ID_STORAGE_KEY),
        )
    }

    fn get_first_token_id_mapper(
        &self,
        sc_address: ManagedAddress,
    ) -> SingleValueMapper<TokenIdentifier, ManagedAddress> {
        SingleValueMapper::<_, _, ManagedAddress>::new_from_address(
            sc_address,
            StorageKey::new(FIRST_TOKEN_ID_STORAGE_KEY),
        )
    }

    fn get_second_token_id_mapper(
        &self,
        sc_address: ManagedAddress,
    ) -> SingleValueMapper<TokenIdentifier, ManagedAddress> {
        SingleValueMapper::<_, _, ManagedAddress>::new_from_address(
            sc_address,
            StorageKey::new(SECOND_TOKEN_ID_STORAGE_KEY),
        )
    }

    fn get_farm_pair_contract_address_mapper(
        &self,
        sc_address: ManagedAddress,
    ) -> SingleValueMapper<ManagedAddress, ManagedAddress> {
        SingleValueMapper::<_, _, ManagedAddress>::new_from_address(
            sc_address,
            StorageKey::new(PAIR_CONTRACT_ADDRESS_STORAGE_KEY),
        )
    }

    fn get_farming_token_id_mapper(
        &self,
        sc_address: ManagedAddress,
    ) -> SingleValueMapper<TokenIdentifier, ManagedAddress> {
        SingleValueMapper::<_, _, ManagedAddress>::new_from_address(
            sc_address,
            StorageKey::new(FARMING_TOKEN_ID_STORAGE_KEY),
        )
    }

    fn get_farm_token_id_mapper(
        &self,
        sc_address: ManagedAddress,
    ) -> SingleValueMapper<TokenIdentifier, ManagedAddress> {
        SingleValueMapper::<_, _, ManagedAddress>::new_from_address(
            sc_address,
            StorageKey::new(FARM_TOKEN_ID_STORAGE_KEY),
        )
    }

    fn get_lp_farm_address_mapper(
        &self,
        sc_address: ManagedAddress,
    ) -> SingleValueMapper<ManagedAddress, ManagedAddress> {
        SingleValueMapper::<_, _, ManagedAddress>::new_from_address(
            sc_address,
            StorageKey::new(LP_FARM_ADDRESS_STORAGE_KEY),
        )
    }

    fn get_staking_farm_address_mapper(
        &self,
        sc_address: ManagedAddress,
    ) -> SingleValueMapper<ManagedAddress, ManagedAddress> {
        SingleValueMapper::<_, _, ManagedAddress>::new_from_address(
            sc_address,
            StorageKey::new(STAKING_FARM_ADDRESS_STORAGE_KEY),
        )
    }

    fn get_farm_state_mapper(
        &self,
        sc_address: ManagedAddress,
    ) -> SingleValueMapper<State, ManagedAddress> {
        SingleValueMapper::<_, _, ManagedAddress>::new_from_address(
            sc_address,
            StorageKey::new(FARM_STATE_STORAGE_KEY),
        )
    }

    fn get_lp_farm_token_id_mapper(
        &self,
        sc_address: ManagedAddress,
    ) -> SingleValueMapper<TokenIdentifier, ManagedAddress> {
        SingleValueMapper::<_, _, ManagedAddress>::new_from_address(
            sc_address,
            StorageKey::new(LP_FARM_TOKEN_ID_STORAGE_KEY),
        )
    }

    fn get_dual_yield_token_id_mapper(
        &self,
        sc_address: ManagedAddress,
    ) -> SingleValueMapper<TokenIdentifier, ManagedAddress> {
        SingleValueMapper::<_, _, ManagedAddress>::new_from_address(
            sc_address,
            StorageKey::new(DUAL_YIELD_TOKEN_ID_STORAGE_KEY),
        )
    }

    fn get_staking_token_id_mapper(
        &self,
        sc_address: ManagedAddress,
    ) -> SingleValueMapper<TokenIdentifier, ManagedAddress> {
        SingleValueMapper::<_, _, ManagedAddress>::new_from_address(
            sc_address,
            StorageKey::new(STAKING_TOKEN_ID_STORAGE_KEY),
        )
    }

    fn get_division_safety_constant_mapper(
        &self,
        sc_address: ManagedAddress,
    ) -> SingleValueMapper<BigUint, ManagedAddress> {
        SingleValueMapper::<_, _, ManagedAddress>::new_from_address(
            sc_address,
            StorageKey::new(DIVISION_SAFETY_CONSTANT_STORAGE_KEY),
        )
    }

    fn get_minimum_farming_epochs_mapper(
        &self,
        sc_address: ManagedAddress,
    ) -> SingleValueMapper<u64, ManagedAddress> {
        SingleValueMapper::<_, _, ManagedAddress>::new_from_address(
            sc_address,
            StorageKey::new(MINIMUM_FARMING_EPOCHS_STORAGE_KEY),
        )
    }
}
