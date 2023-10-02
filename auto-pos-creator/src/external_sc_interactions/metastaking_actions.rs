pub use farm_staking_proxy::proxy_actions::stake::ProxyTrait as OtherProxyTrait2;
pub use farm_staking_proxy::proxy_actions::unstake::ProxyTrait as OtherProxyTrait;
use farm_staking_proxy::result_types::{StakeProxyResult, UnstakeResult};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait MetastakingActionsModule {
    fn call_metastaking_stake(
        &self,
        sc_address: ManagedAddress,
        user: ManagedAddress,
        lp_farm_tokens: EsdtTokenPayment,
    ) -> StakeProxyResult<Self::Api> {
        self.metastaking_proxy(sc_address)
            .stake_farm_tokens(user)
            .with_esdt_transfer(lp_farm_tokens)
            .execute_on_dest_context()
    }

    fn call_metastaking_unstake(
        &self,
        sc_address: ManagedAddress,
        user: ManagedAddress,
        dual_yield_tokens: EsdtTokenPayment,
        first_token_min_amount_out: BigUint,
        second_token_min_amont_out: BigUint,
    ) -> UnstakeResult<Self::Api> {
        self.metastaking_proxy(sc_address)
            .unstake_farm_tokens(
                first_token_min_amount_out,
                second_token_min_amont_out,
                dual_yield_tokens.amount.clone(),
                user,
            )
            .with_esdt_transfer(dual_yield_tokens)
            .execute_on_dest_context()
    }

    #[proxy]
    fn metastaking_proxy(&self, sc_address: ManagedAddress)
        -> farm_staking_proxy::Proxy<Self::Api>;
}
