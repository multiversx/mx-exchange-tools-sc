use common_structs::PaymentsVec;
use energy_factory::token_merging::ProxyTrait as _;

use crate::user_tokens::user_rewards::RewardsWrapper;

use mergeable::Mergeable;

elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait LockedTokenMergingModule: energy_query::EnergyQueryModule {
    fn merge_wrapped_locked_tokens(
        &self,
        user: ManagedAddress,
        wrapper: &mut RewardsWrapper<Self::Api>,
        new_locked_tokens: EsdtTokenPayment,
    ) {
        let opt_existing_fees = wrapper.opt_locked_tokens.as_mut();
        if opt_existing_fees.is_none() {
            wrapper.opt_locked_tokens = Some(new_locked_tokens);
            return;
        }

        let existing_fees = unsafe { opt_existing_fees.unwrap_unchecked() };
        if existing_fees.can_merge_with(&new_locked_tokens) {
            existing_fees.amount += new_locked_tokens.amount;
        } else {
            let mut locked_token_payments = PaymentsVec::from_single_item(existing_fees.clone());
            locked_token_payments.push(new_locked_tokens);
            wrapper.opt_locked_tokens = self.merge_locked_tokens(user, locked_token_payments);
        }
    }

    fn merge_locked_tokens(
        &self,
        user: ManagedAddress,
        locked_tokens: PaymentsVec<Self::Api>,
    ) -> Option<EsdtTokenPayment> {
        if locked_tokens.is_empty() {
            return None;
        }
        if locked_tokens.len() == 1 {
            return Some(locked_tokens.get(0));
        }

        let energy_factory_address = self.energy_factory_address().get();
        let new_token = self
            .energy_factory_proxy(energy_factory_address)
            .merge_tokens_endpoint(user)
            .execute_on_dest_context();

        Some(new_token)
    }
}
