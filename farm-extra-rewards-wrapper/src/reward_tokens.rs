use common_structs::PaymentsVec;

use crate::common::payments_wrapper::PaymentsWrapper;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait RewardTokensModule: permissions_module::PermissionsModule {
    /// Deposit tokens and add them to the whitelist
    #[payable("*")]
    #[endpoint(depositRewardTokens)]
    fn deposit_reward_tokens(&self) {
        self.require_caller_has_owner_or_admin_permissions();

        let payments = self.call_value().all_esdt_transfers().clone_value();
        let current_timestamp = self.blockchain().get_block_timestamp();
        let mut tokens_mapper = self.reward_tokens();
        for payment in &payments {
            require!(payment.token_nonce == 0, "Only fungible tokens accepted");

            self.reward_capacity(&payment.token_identifier)
                .update(|total| *total += &payment.amount);

            let is_new = tokens_mapper.insert(payment.token_identifier.clone());
            if is_new {
                self.token_addition_timestamp(&payment.token_identifier)
                    .set(current_timestamp);
            }
        }
    }

    /// Withdraw all remaining given tokens and remove them from the whitelist
    #[endpoint(withdrawRewardTokens)]
    fn withdraw_reward_tokens(
        &self,
        tokens: MultiValueEncoded<TokenIdentifier>,
    ) -> PaymentsVec<Self::Api> {
        self.require_caller_has_owner_or_admin_permissions();

        let mut output_payments = PaymentsWrapper::new();
        let mut tokens_mapper = self.reward_tokens();
        for token in tokens {
            let _ = tokens_mapper.swap_remove(&token);
            self.token_addition_timestamp(&token).clear();

            let accumulated_rewards = self.accumulated_rewards(&token).take();
            let capacity = self.reward_capacity(&token).take();
            let remaining_tokens = capacity - accumulated_rewards;

            let payment = EsdtTokenPayment::new(token, 0, remaining_tokens);
            output_payments.push(payment);
        }

        let caller = self.blockchain().get_caller();
        output_payments.send_to(&caller);

        output_payments.into_payments()
    }

    #[view(getRewardTokens)]
    #[storage_mapper("rewTokens")]
    fn reward_tokens(&self) -> UnorderedSetMapper<TokenIdentifier>;

    #[view(getTokenAdditionTimestamp)]
    #[storage_mapper("tokenAddTs")]
    fn token_addition_timestamp(&self, token_id: &TokenIdentifier) -> SingleValueMapper<u64>;

    #[storage_mapper("accRew")]
    fn accumulated_rewards(&self, token_id: &TokenIdentifier) -> SingleValueMapper<BigUint>;

    #[storage_mapper("rewCap")]
    fn reward_capacity(&self, token_id: &TokenIdentifier) -> SingleValueMapper<BigUint>;
}
