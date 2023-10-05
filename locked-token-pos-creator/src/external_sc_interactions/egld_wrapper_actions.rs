multiversx_sc::imports!();

mod egld_wrapper_proxy {
    multiversx_sc::imports!();

    #[multiversx_sc::proxy]
    pub trait EgldWrapperProxy {
        #[payable("EGLD")]
        #[endpoint(wrapEgld)]
        fn wrap_egld(&self) -> EsdtTokenPayment;
    }
}

#[multiversx_sc::module]
pub trait EgldWrapperActionsModule {
    fn call_wrap_egld(&self, egld_amount: BigUint) -> EsdtTokenPayment {
        let wrapper_sc_address = self.egld_wrapper_sc_address().get();
        self.egld_wrapper_proxy(wrapper_sc_address)
            .wrap_egld()
            .with_egld_transfer(egld_amount)
            .execute_on_dest_context()
    }

    #[storage_mapper("wegldTokenId")]
    fn wegld_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("egldWrapperScAddress")]
    fn egld_wrapper_sc_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[proxy]
    fn egld_wrapper_proxy(
        &self,
        sc_address: ManagedAddress,
    ) -> egld_wrapper_proxy::Proxy<Self::Api>;
}
