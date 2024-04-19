multiversx_sc::imports!();

use common_structs::PaymentsVec;
pub use farm_staking_proxy::proxy_actions::stake::ProxyTrait as OtherProxyTrait2;
pub use farm_staking_proxy::proxy_actions::unstake::ProxyTrait as OtherProxyTrait;
use farm_staking_proxy::result_types::{StakeProxyResult, UnstakeResult};

use crate::multi_contract_interactions::exit_pos::MetastakingExitArgs;

#[multiversx_sc::module]
pub trait MetastakingActionsModule {
    fn call_metastaking_stake(
        &self,
        sc_address: ManagedAddress,
        user: ManagedAddress,
        payments: PaymentsVec<Self::Api>,
    ) -> StakeProxyResult<Self::Api> {
        self.metastaking_proxy(sc_address)
            .stake_farm_tokens(user)
            .with_multi_token_transfer(payments)
            .execute_on_dest_context()
    }

    fn call_metastaking_unstake(
        &self,
        args: MetastakingExitArgs<Self::Api>,
    ) -> UnstakeResult<Self::Api> {
        self.metastaking_proxy(args.ms_address)
            .unstake_farm_tokens(
                args.first_token_min_amount_out,
                args.second_token_min_amont_out,
                args.user,
            )
            .with_esdt_transfer(args.ms_tokens)
            .execute_on_dest_context()
    }

    #[proxy]
    fn metastaking_proxy(&self, sc_address: ManagedAddress)
        -> farm_staking_proxy::Proxy<Self::Api>;
}
