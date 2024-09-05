multiversx_sc::imports!();

use common_structs::{Epoch, PaymentsVec};
use proxies::energy_factory_proxy;

#[multiversx_sc::module]
pub trait LockedTokenModule: energy_query::EnergyQueryModule {
    fn lock_tokens(&self, payment: EsdtTokenPayment, epoch: Epoch) -> EsdtTokenPayment {
        let energy_factory_address = self.energy_factory_address().get();

        self.tx()
            .to(&energy_factory_address)
            .typed(energy_factory_proxy::SimpleLockEnergyProxy)
            .lock_tokens_endpoint(epoch, OptionalValue::<ManagedAddress>::None)
            .payment(payment)
            .returns(ReturnsResult)
            .sync_call()
    }

    fn merge_locked_tokens(&self, locked_tokens: PaymentsVec<Self::Api>) -> EsdtTokenPayment {
        if locked_tokens.len() == 1 {
            return locked_tokens.get(0);
        }

        let energy_factory_address = self.energy_factory_address().get();
        self.tx()
            .to(&energy_factory_address)
            .typed(energy_factory_proxy::SimpleLockEnergyProxy)
            .merge_tokens_endpoint(OptionalValue::<ManagedAddress>::None)
            .payment(locked_tokens)
            .returns(ReturnsResult)
            .sync_call()
    }

    fn wrap_locked_token(&self, payment: EsdtTokenPayment<Self::Api>) -> EsdtTokenPayment {
        let sc_address = self.locked_token_wrapper_sc_address().get();
        self.locked_token_wrapper_proxy(sc_address)
            .wrap_locked_token_endpoint()
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
