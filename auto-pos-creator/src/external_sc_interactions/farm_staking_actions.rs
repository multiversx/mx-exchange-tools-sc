multiversx_sc::imports!();

use common_structs::PaymentsVec;
use farm::EnterFarmResultType;
use farm_staking::stake_farm::ProxyTrait as OtherProxyTrait;

#[multiversx_sc::module]
pub trait FarmStakingActionsModule: read_external_storage::ReadExternalStorageModule {
    fn call_farm_staking_stake(
        &self,
        sc_address: ManagedAddress,
        user: ManagedAddress,
        tokens: PaymentsVec<Self::Api>,
    ) -> EnterFarmResultType<Self::Api> {
        self.farm_staking_proxy(sc_address)
            .stake_farm_endpoint(OptionalValue::Some(user))
            .with_multi_token_transfer(tokens)
            .execute_on_dest_context()
    }

    fn get_farm_staking_farming_token_id(&self, sc_address: ManagedAddress) -> TokenIdentifier {
        self.get_farming_token_id_mapper(sc_address).get()
    }

    #[proxy]
    fn farm_staking_proxy(&self, sc_address: ManagedAddress) -> farm_staking::Proxy<Self::Api>;

    #[storage_mapper("farmStakingAddressForToken")]
    fn farm_staking_address_for_token(
        &self,
        token_id: &TokenIdentifier,
    ) -> SingleValueMapper<ManagedAddress>;
}
