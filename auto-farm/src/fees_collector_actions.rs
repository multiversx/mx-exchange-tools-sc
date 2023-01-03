use common_structs::PaymentsVec;

use crate::user_rewards::UniquePayments;

elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait FeesCollectorActionsModule:
    crate::common_storage::CommonStorageModule
    + crate::user_rewards::UserRewardsModule
    + crate::fees::FeesModule
    + utils::UtilsModule
    + energy_query::EnergyQueryModule
    + crate::locked_token_merging::LockedTokenMergingModule
    + lkmex_transfer::energy_transfer::EnergyTransferModule
    + legacy_token_decode_module::LegacyTokenDecodeModule
{
    #[endpoint(claimFeesCollectorRewards)]
    fn claim_fees_collector_rewards(&self, user: ManagedAddress) {
        self.require_caller_proxy_claim_address();

        let mut rewards = self.call_fees_collector_claim(user.clone());
        let rewards_len = rewards.len();
        if rewards_len == 0 {
            return;
        }

        // locked token rewards, if any, are always in the last position
        let locked_token_id = self.get_locked_token_id();
        let last_payment = rewards.get(rewards_len - 1);
        let mut locked_tokens = UniquePayments::new();
        if last_payment.token_identifier == locked_token_id {
            locked_tokens.add_payment(last_payment);
            rewards.remove(rewards_len - 1);
        }

        let merged_rewards = UniquePayments::new_from_payments(rewards);
        let user_id = self.user_ids().get_id(&user);
        self.add_user_rewards(user, user_id, locked_tokens, merged_rewards);
    }

    fn call_fees_collector_claim(&self, user: ManagedAddress) -> PaymentsVec<Self::Api> {
        let sc_address = self.fees_collector_sc_address().get();
        self.fees_collector_proxy(sc_address)
            .claim_rewards(user)
            .execute_on_dest_context()
    }

    #[proxy]
    fn fees_collector_proxy(&self, sc_address: ManagedAddress) -> fees_collector::Proxy<Self::Api>;

    #[storage_mapper("feesCollectorScAddress")]
    fn fees_collector_sc_address(&self) -> SingleValueMapper<ManagedAddress>;
}
