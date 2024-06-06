use common_structs::Epoch;
use energy_factory::energy_factory_proxy;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait EnergyFactoryActionsModule: energy_query::EnergyQueryModule {
    fn call_lock_virtual(
        &self,
        payment: EsdtTokenPayment,
        lock_epochs: Epoch,
        user: ManagedAddress,
    ) -> EsdtTokenPayment {
        let energy_factory_address = self.energy_factory_address().get();
        let own_address = self.blockchain().get_sc_address();

        let locked_tokens = self
            .tx()
            .to(&energy_factory_address)
            .typed(energy_factory_proxy::SimpleLockEnergyProxy)
            .lock_virtual(
                payment.token_identifier.clone(),
                payment.amount.clone(),
                lock_epochs,
                own_address,
                user,
            )
            .returns(ReturnsResult)
            .sync_call();

        self.send()
            .esdt_local_burn(&payment.token_identifier, 0, &payment.amount);

        locked_tokens
    }
}
