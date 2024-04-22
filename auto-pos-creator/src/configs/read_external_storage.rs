use multiversx_sc::storage::StorageKey;

multiversx_sc::imports!();

pub static LP_TOKEN_ID_STORAGE_KEY: &[u8] = b"lpTokenIdentifier";
pub static FIRST_TOKEN_ID_STORAGE_KEY: &[u8] = b"first_token_id";
pub static SECOND_TOKEN_ID_STORAGE_KEY: &[u8] = b"second_token_id";
pub static PAIR_CONTRACT_ADDRESS_STORAGE_KEY: &[u8] = b"pair_contract_address";
pub static FARMING_TOKEN_ID_STORAGE_KEY: &[u8] = b"farming_token_id";
pub static LP_FARM_ADDRESS_STORAGE_KEY: &[u8] = b"lpFarmAddress";

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

    fn get_farm_staking_farming_token_id_mapper(
        &self,
        sc_address: ManagedAddress,
    ) -> SingleValueMapper<TokenIdentifier, ManagedAddress> {
        SingleValueMapper::<_, _, ManagedAddress>::new_from_address(
            sc_address,
            StorageKey::new(FARMING_TOKEN_ID_STORAGE_KEY),
        )
    }

    fn get_lp_farm_contract_address_mapper(
        &self,
        metastaking_address: ManagedAddress,
    ) -> SingleValueMapper<ManagedAddress, ManagedAddress> {
        SingleValueMapper::<_, _, ManagedAddress>::new_from_address(
            metastaking_address,
            StorageKey::new(LP_FARM_ADDRESS_STORAGE_KEY),
        )
    }
}
