multiversx_sc::imports!();

use common_structs::PaymentsVec;
pub use farm_staking_proxy::proxy_actions::claim::ProxyTrait as _;
use farm_staking_proxy::result_types::ClaimDualYieldResult;

use crate::common::rewards_wrapper::RewardsWrapper;

#[multiversx_sc::module]
pub trait MetastakingActionsModule:
    read_external_storage::ReadExternalStorageModule
    + crate::common::common_storage::CommonStorageModule
    + crate::whitelists::metastaking_whitelist::MetastakingWhitelistModule
    + crate::external_storage_read::metastaking_storage_read::MetastakingStorageReadModule
    + crate::user_tokens::user_metastaking_tokens::UserMetastakingTokensModule
    + crate::user_tokens::user_rewards::UserRewardsModule
    + crate::user_tokens::withdraw_tokens::WithdrawTokensModule
    + crate::fees::FeesModule
    + crate::events::EventsModule
    + utils::UtilsModule
    + energy_query::EnergyQueryModule
    + crate::external_sc_interactions::locked_token_merging::LockedTokenMergingModule
    + lkmex_transfer::energy_transfer::EnergyTransferModule
    + legacy_token_decode_module::LegacyTokenDecodeModule
{
    fn claim_all_metastaking_rewards(
        &self,
        user: &ManagedAddress,
        user_id: AddressId,
        rew_wrapper: &mut RewardsWrapper<Self::Api>,
    ) {
        let ms_mapper = self.metastaking_ids();
        let user_tokens_mapper = self.user_metastaking_tokens(user_id);
        let user_dual_yield_tokens = user_tokens_mapper.get();
        if user_dual_yield_tokens.is_empty() {
            return;
        }

        let mut new_user_dual_yield_tokens = PaymentsVec::new();
        for dual_yield_token in &user_dual_yield_tokens {
            let ms_id = self
                .metastaking_for_dual_yield_token(&dual_yield_token.token_identifier)
                .get();
            let opt_ms_addr = ms_mapper.get_address(ms_id);
            if opt_ms_addr.is_none() {
                new_user_dual_yield_tokens.push(dual_yield_token);
                continue;
            }

            let ms_addr = unsafe { opt_ms_addr.unwrap_unchecked() };
            let claim_result = self.call_metastaking_claim(ms_addr, user.clone(), dual_yield_token);
            new_user_dual_yield_tokens.push(claim_result.new_dual_yield_tokens);

            rew_wrapper.add_tokens(claim_result.lp_farm_rewards);
            rew_wrapper.add_tokens(claim_result.staking_farm_rewards);
        }

        user_tokens_mapper.set(&new_user_dual_yield_tokens);
    }

    fn call_metastaking_claim(
        &self,
        ms_address: ManagedAddress,
        user: ManagedAddress,
        dual_yield_token: EsdtTokenPayment,
    ) -> ClaimDualYieldResult<Self::Api> {
        self.metastaking_proxy(ms_address)
            .claim_dual_yield_endpoint(OptionalValue::Some(user))
            .with_esdt_transfer(dual_yield_token)
            .execute_on_dest_context()
    }

    #[proxy]
    fn metastaking_proxy(&self, sc_address: ManagedAddress)
        -> farm_staking_proxy::Proxy<Self::Api>;
}
