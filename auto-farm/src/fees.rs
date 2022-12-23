use common_structs::PaymentsVec;

use crate::{
    common_storage::MAX_PERCENTAGE,
    user_rewards::{RewardsWrapper, UniquePayments},
};

elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait FeesModule:
    crate::common_storage::CommonStorageModule
    + crate::locked_token_merging::LockedTokenMergingModule
    + lkmex_transfer::energy_transfer::EnergyTransferModule
    + legacy_token_decode_module::LegacyTokenDecodeModule
    + energy_query::EnergyQueryModule
    + utils::UtilsModule
{
    fn take_fees(&self, user: ManagedAddress, rewards_wrapper: &mut RewardsWrapper<Self::Api>) {
        let accumulated_fees_mapper = self.accumulated_fees();
        let mut fees_wrapper = if !accumulated_fees_mapper.is_empty() {
            accumulated_fees_mapper.get()
        } else {
            RewardsWrapper::default()
        };

        let fee_percentage = self.fee_percentage().get();
        self.add_locked_token_fees(user, &mut fees_wrapper, rewards_wrapper, fee_percentage);

        let other_tokens = rewards_wrapper.other_tokens.clone().into_payments();
        let mut remaining_user_tokens = PaymentsVec::new();
        for i in 0..other_tokens.len() {
            let mut current_token = other_tokens.get(i);
            let fee_tokens = self.deduct_single_fee(&mut current_token, fee_percentage);
            fees_wrapper.other_tokens.add_payment(fee_tokens);

            if current_token.amount > 0 {
                remaining_user_tokens.push(current_token);
            }
        }

        rewards_wrapper.other_tokens = UniquePayments::new_from_payments(remaining_user_tokens);
        accumulated_fees_mapper.set(fees_wrapper);
    }

    fn add_locked_token_fees(
        &self,
        user: ManagedAddress,
        fees_wrapper: &mut RewardsWrapper<Self::Api>,
        rewards_wrapper: &mut RewardsWrapper<Self::Api>,
        fee_percentage: u64,
    ) {
        let opt_new_locked_tokens = rewards_wrapper.opt_locked_tokens.as_mut();
        if opt_new_locked_tokens.is_none() {
            return;
        }

        let new_locked_tokens = unsafe { opt_new_locked_tokens.unwrap_unchecked() };
        let fee_tokens = self.deduct_single_fee(new_locked_tokens, fee_percentage);
        if fee_tokens.amount == 0 {
            return;
        }
        if new_locked_tokens.amount == 0 {
            rewards_wrapper.opt_locked_tokens = None;
        }

        let proxy_addr = self.proxy_claim_address().get();
        self.deduct_energy_from_sender(user, &ManagedVec::from_single_item(fee_tokens.clone()));
        self.add_energy_to_destination(
            proxy_addr.clone(),
            &ManagedVec::from_single_item(fee_tokens.clone()),
        );

        self.merge_wrapped_locked_tokens(proxy_addr, fees_wrapper, fee_tokens);
    }

    fn deduct_single_fee(
        &self,
        payment: &mut EsdtTokenPayment,
        fee_percentage: u64,
    ) -> EsdtTokenPayment {
        let fee_amount = self.calculate_fee_amount(&payment.amount, fee_percentage);
        payment.amount -= &fee_amount;

        EsdtTokenPayment::new(
            payment.token_identifier.clone(),
            payment.token_nonce,
            fee_amount,
        )
    }

    fn calculate_fee_amount(&self, payment_amount: &BigUint, fee_percentage: u64) -> BigUint {
        payment_amount * fee_percentage / MAX_PERCENTAGE
    }

    #[view(getFeePercentage)]
    #[storage_mapper("feePercentage")]
    fn fee_percentage(&self) -> SingleValueMapper<u64>;

    #[view(getAccumulatedFees)]
    #[storage_mapper("accumulatedFees")]
    fn accumulated_fees(&self) -> SingleValueMapper<RewardsWrapper<Self::Api>>;
}
