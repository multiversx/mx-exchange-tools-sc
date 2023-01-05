use common_structs::PaymentsVec;
use farm_staking_proxy::result_types::ClaimDualYieldResult;

use crate::common::unique_payments::UniquePayments;

elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait MetastakingActionsModule:
    crate::common::common_storage::CommonStorageModule
    + crate::whitelists::metastaking_whitelist::MetastakingWhitelistModule
    + crate::external_storage_read::metastaking_storage_read::MetastakingStorageReadModule
    + crate::user_tokens::user_metastaking_tokens::UserMetastakingTokensModule
    + crate::user_tokens::user_rewards::UserRewardsModule
    + crate::fees::FeesModule
    + utils::UtilsModule
    + energy_query::EnergyQueryModule
    + crate::external_sc_interactions::locked_token_merging::LockedTokenMergingModule
    + lkmex_transfer::energy_transfer::EnergyTransferModule
    + legacy_token_decode_module::LegacyTokenDecodeModule
{
    /// Arg: user to claim rewards for
    #[endpoint(claimAllMetastakingRewards)]
    fn claim_all_metastaking_rewards(&self, user: ManagedAddress) {
        self.require_caller_proxy_claim_address();

        let ms_mapper = self.metastaking_ids();
        let user_id = self.user_ids().get_id_non_zero(&user);

        let locked_token_id = self.get_locked_token_id();
        let user_tokens_mapper = self.user_metastaking_tokens(user_id);
        let user_dual_yield_tokens = user_tokens_mapper.get();

        let mut new_user_dual_yield_tokens = PaymentsVec::new();
        let mut locked_rewards = UniquePayments::new();
        let mut other_token_rewards = UniquePayments::new();
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

            let lp_rewards = claim_result.lp_farm_rewards;
            if lp_rewards.token_identifier == locked_token_id {
                locked_rewards.add_payment(lp_rewards);
            } else {
                other_token_rewards.add_payment(lp_rewards);
            }

            let staking_rewards = claim_result.staking_farm_rewards;
            if staking_rewards.token_identifier == locked_token_id {
                locked_rewards.add_payment(staking_rewards);
            } else {
                other_token_rewards.add_payment(staking_rewards);
            }
        }

        self.add_user_rewards(user, user_id, locked_rewards, other_token_rewards);

        user_tokens_mapper.set(&new_user_dual_yield_tokens);
    }

    fn call_metastaking_claim(
        &self,
        ms_address: ManagedAddress,
        user: ManagedAddress,
        dual_yield_token: EsdtTokenPayment,
    ) -> ClaimDualYieldResult<Self::Api> {
        self.metastaking_proxy(ms_address)
            .claim_dual_yield(user)
            .with_esdt_transfer(dual_yield_token)
            .execute_on_dest_context()
    }

    #[proxy]
    fn metastaking_proxy(&self, sc_address: ManagedAddress)
        -> farm_staking_proxy::Proxy<Self::Api>;
}
