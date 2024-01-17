multiversx_sc::imports!();

use common_structs::PaymentsVec;
use locked_token_wrapper::wrapped_token;

use crate::common::{rewards_wrapper::RewardsWrapper, unique_payments::UniquePayments};

#[multiversx_sc::module]
pub trait FeesCollectorInteractionsModule:
    crate::external_sc_interactions::farm_actions::FarmActionsModule
    + crate::external_sc_interactions::energy_dao_config::EnergyDAOConfigModule
    + crate::external_sc_interactions::locked_token_actions::LockedTokenModule
    + utils::UtilsModule
    + permissions_module::PermissionsModule
    + energy_query::EnergyQueryModule
    + lkmex_transfer::energy_transfer::EnergyTransferModule
    + legacy_token_decode_module::LegacyTokenDecodeModule
    + wrapped_token::WrappedTokenModule
    + simple_lock::token_attributes::TokenAttributesModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    #[endpoint(claimFeesCollectorRewards)]
    fn claim_fees_collector_rewards(&self) {
        let mut rewards = self.call_fees_collector_claim();
        let rewards_len = rewards.len();
        if rewards_len == 0 {
            return;
        }

        // tokens from the fees collector are kept by the contract
        let collected_fees_mapper = self.collected_fees();
        let mut new_collected_fees = if collected_fees_mapper.is_empty() {
            let locked_token_id = self.get_locked_token_id();
            RewardsWrapper::new(locked_token_id)
        } else {
            collected_fees_mapper.get()
        };

        // locked token rewards, if any, are always in the last position
        let last_payment = rewards.get(rewards_len - 1);
        if &last_payment.token_identifier == new_collected_fees.get_locked_token_id() {
            let mut fees_payments = new_collected_fees.locked_tokens.into_payments();
            fees_payments.push(last_payment);
            let new_locked_fee = self.merge_locked_tokens(fees_payments);
            new_collected_fees.locked_tokens = UniquePayments::new();
            new_collected_fees.add_tokens(new_locked_fee);
            rewards.remove(rewards_len - 1);
        }

        for rew in &rewards {
            new_collected_fees.add_tokens(rew);
        }
        collected_fees_mapper.set(new_collected_fees);
    }

    fn call_fees_collector_claim(&self) -> PaymentsVec<Self::Api> {
        let sc_address = self.fees_collector_sc_address().get();
        self.fees_collector_proxy(sc_address)
            .claim_rewards_endpoint(OptionalValue::<ManagedAddress>::None)
            .execute_on_dest_context()
    }

    #[proxy]
    fn fees_collector_proxy(&self, sc_address: ManagedAddress) -> fees_collector::Proxy<Self::Api>;

    #[storage_mapper("feesCollectorScAddress")]
    fn fees_collector_sc_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("collectedFees")]
    fn collected_fees(&self) -> SingleValueMapper<RewardsWrapper<Self::Api>>;
}
