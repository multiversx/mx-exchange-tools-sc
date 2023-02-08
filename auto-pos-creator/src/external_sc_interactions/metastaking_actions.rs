use farm_staking_proxy::result_types::{StakeProxyResult, UnstakeResult};
pub use farm_staking_proxy::proxy_actions::unstake::ProxyTrait as OtherProxyTrait;
pub use farm_staking_proxy::proxy_actions::stake::ProxyTrait as OtherProxyTrait2;

use super::pair_actions::MIN_AMOUNT_OUT;

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
    ) -> UnstakeResult<Self::Api> {
        self.metastaking_proxy(sc_address)
            .unstake_farm_tokens(
                MIN_AMOUNT_OUT,
                MIN_AMOUNT_OUT,
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
