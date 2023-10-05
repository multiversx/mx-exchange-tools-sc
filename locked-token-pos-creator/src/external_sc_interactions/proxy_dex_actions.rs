use proxy_dex::{proxy_farm::ProxyTrait as _, proxy_pair::ProxyTrait as _};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TypeAbi, TopDecode, TopEncode)]
pub struct AddLiquidityProxyResult<M: ManagedTypeApi> {
    pub wrapped_lp_token: EsdtTokenPayment<M>,
    pub locked_token_leftover: EsdtTokenPayment<M>,
    pub wegld_leftover: EsdtTokenPayment<M>,
}

#[multiversx_sc::module]
pub trait ProxyDexActionsModule {
    fn call_add_liquidity_proxy(
        &self,
        payments: ManagedVec<EsdtTokenPayment>,
        pair_address: ManagedAddress,
        first_token_amount_min: BigUint,
        second_token_amount_min: BigUint,
    ) -> AddLiquidityProxyResult<Self::Api> {
        let proxy_dex_address = self.proxy_dex_address().get();
        let output_payments: MultiValueEncoded<EsdtTokenPayment> = self
            .proxy_dex_proxy(proxy_dex_address)
            .add_liquidity_proxy(
                pair_address,
                first_token_amount_min,
                second_token_amount_min,
            )
            .with_multi_token_transfer(payments)
            .execute_on_dest_context();

        let output_payments_vec = output_payments.to_vec();

        AddLiquidityProxyResult {
            wrapped_lp_token: output_payments_vec.get(0),
            locked_token_leftover: output_payments_vec.get(1),
            wegld_leftover: output_payments_vec.get(2),
        }
    }

    fn call_enter_farm_proxy(
        &self,
        payment: EsdtTokenPayment,
        farm_address: ManagedAddress,
    ) -> EsdtTokenPayment {
        let proxy_dex_address = self.proxy_dex_address().get();
        self.proxy_dex_proxy(proxy_dex_address)
            .enter_farm_proxy_endpoint(farm_address)
            .with_esdt_transfer(payment)
            .execute_on_dest_context()
    }

    #[storage_mapper("proxyDexAddress")]
    fn proxy_dex_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[proxy]
    fn proxy_dex_proxy(&self, sc_address: ManagedAddress) -> proxy_dex::Proxy<Self::Api>;
}
