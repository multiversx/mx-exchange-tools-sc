multiversx_sc::imports!();

use router::{factory::ProxyTrait as _, multi_pair_swap::ProxyTrait as _};

pub type SwapOperationType<M> =
    MultiValue4<ManagedAddress<M>, ManagedBuffer<M>, TokenIdentifier<M>, BigUint<M>>;

#[multiversx_sc::module]
pub trait RouterActionsModule {
    fn check_router_pair(
        &self,
        pair_address: ManagedAddress,
        first_token_id: TokenIdentifier,
        second_token_id: TokenIdentifier,
    ) {
        let router_address = self.router_address().get();
        let router_pair_address: ManagedAddress = self
            .router_proxy(router_address)
            .get_pair(first_token_id, second_token_id)
            .execute_on_dest_context();

        require!(
            pair_address == router_pair_address,
            "Pair address not found in router"
        );
    }

    fn call_router_swap(
        &self,
        input_tokens: EsdtTokenPayment,
        swap_operations: MultiValueEncoded<SwapOperationType<Self::Api>>,
    ) -> EsdtTokenPayment {
        let router_address = self.router_address().get();

        let ((), back_transfers) = self
            .router_proxy(router_address)
            .multi_pair_swap(swap_operations)
            .with_esdt_transfer(input_tokens)
            .execute_on_dest_context_with_back_transfers();

        require!(
            back_transfers.esdt_payments.len() == 1,
            "Wrong number of output tokens. Use only fixed input swaps"
        );

        back_transfers.esdt_payments.get(0)
    }

    #[proxy]
    fn router_proxy(&self, sc_address: ManagedAddress) -> router::Proxy<Self::Api>;

    #[storage_mapper("routerAddress")]
    fn router_address(&self) -> SingleValueMapper<ManagedAddress>;
}
