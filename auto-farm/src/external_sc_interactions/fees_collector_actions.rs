use common_structs::PaymentsVec;

use crate::common::rewards_wrapper::RewardsWrapper;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait FeesCollectorActionsModule:
    crate::common::common_storage::CommonStorageModule
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
    fn claim_fees_collector_rewards(
        &self,
        user: &ManagedAddress,
        rew_wrapper: &mut RewardsWrapper<Self::Api>,
    ) {
        let mut rewards = self.call_fees_collector_claim(user.clone());
        let rewards_len = rewards.len();
        if rewards_len == 0 {
            return;
        }

        // locked token rewards, if any, are always in the last position
        let last_payment = rewards.get(rewards_len - 1);
        if &last_payment.token_identifier == rew_wrapper.get_locked_token_id() {
            rew_wrapper.locked_tokens.add_payment(last_payment);
            rewards.remove(rewards_len - 1);
        }

        for rew in &rewards {
            rew_wrapper.other_tokens.add_payment(rew);
        }
    }

    fn call_fees_collector_claim(&self, user: ManagedAddress) -> PaymentsVec<Self::Api> {
        let sc_address = self.fees_collector_sc_address().get();
        self.fees_collector_proxy(sc_address)
            .claim_rewards_endpoint(OptionalValue::Some(user))
            .execute_on_dest_context()
    }

    #[proxy]
    fn fees_collector_proxy(&self, sc_address: ManagedAddress) -> fees_collector::Proxy<Self::Api>;

    #[storage_mapper("feesCollectorScAddress")]
    fn fees_collector_sc_address(&self) -> SingleValueMapper<ManagedAddress>;
}
