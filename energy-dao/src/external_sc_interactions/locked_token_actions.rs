multiversx_sc::imports!();

use common_structs::{Epoch, PaymentsVec};
use energy_factory::token_merging::ProxyTrait as _;
use energy_factory::ProxyTrait as _;

#[multiversx_sc::module]
pub trait LockedTokenModule: energy_query::EnergyQueryModule {
    fn lock_tokens(&self, payment: EsdtTokenPayment, epoch: Epoch) -> EsdtTokenPayment {
        let energy_factory_address = self.energy_factory_address().get();
        self.energy_factory_proxy(energy_factory_address)
            .lock_tokens_endpoint(epoch, OptionalValue::<ManagedAddress>::None)
            .with_egld_or_single_esdt_transfer(payment)
            .execute_on_dest_context()
    }

    fn merge_locked_tokens(&self, locked_tokens: PaymentsVec<Self::Api>) -> EsdtTokenPayment {
        if locked_tokens.len() == 1 {
            return locked_tokens.get(0);
        }

        let energy_factory_address = self.energy_factory_address().get();
        self.energy_factory_proxy(energy_factory_address)
            .merge_tokens_endpoint(OptionalValue::<ManagedAddress>::None)
            .with_multi_token_transfer(locked_tokens)
            .execute_on_dest_context()
    }

    fn wrap_locked_token(&self, payment: EsdtTokenPayment<Self::Api>) -> EsdtTokenPayment {
        let sc_address = self.locked_token_wrapper_sc_address().get();
        self.energy_factory_proxy(sc_address)
            .merge_tokens_endpoint(OptionalValue::<ManagedAddress>::None)
            .with_esdt_transfer(payment)
            .execute_on_dest_context()
    }

    #[proxy]
    fn locked_token_wrapper_proxy(
        &self,
        sc_address: ManagedAddress,
    ) -> locked_token_wrapper::Proxy<Self::Api>;

    #[storage_mapper("lockedTokenWrapperScAddress")]
    fn locked_token_wrapper_sc_address(&self) -> SingleValueMapper<ManagedAddress>;
}
