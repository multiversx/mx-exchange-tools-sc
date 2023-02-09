use common_structs::PaymentsVec;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait RewardTokensModule {
    /// Deposit tokens and add them to the whitelist
    #[only_owner]
    #[payable("*")]
    #[endpoint(depositRewardTokens)]
    fn deposit_reward_tokens(&self) {
        let payments = self.call_value().all_esdt_transfers();
        let mut tokens_mapper = self.reward_tokens();
        for payment in &payments {
            self.reward_capacity(&payment.token_identifier)
                .update(|total| *total += &payment.amount);

            let _ = tokens_mapper.insert(payment.token_identifier);
        }
    }

    /// Withdraw all remaining given tokens and remove them from the whitelist
    #[only_owner]
    #[endpoint(withdrawRewardTokens)]
    fn withdraw_reward_tokens(
        &self,
        tokens: MultiValueEncoded<TokenIdentifier>,
    ) -> PaymentsVec<Self::Api> {
        let mut output_payments = PaymentsVec::new();
        let mut tokens_mapper = self.reward_tokens();
        for token in tokens {
            let accumulated_rewards = self.accumulated_rewards(&token).take();
            let capacity = self.reward_capacity(&token).take();
            let remaining_tokens = capacity - accumulated_rewards;
            if remaining_tokens == 0 {
                continue;
            }

            let _ = tokens_mapper.swap_remove(&token);

            let payment = EsdtTokenPayment::new(token, 0, remaining_tokens);
            output_payments.push(payment);
        }

        let caller = self.blockchain().get_caller();
        if !output_payments.is_empty() {
            self.send().direct_multi(&caller, &output_payments);
        }

        output_payments
    }

    #[view(getRewardTokens)]
    #[storage_mapper("rewardTokens")]
    fn reward_tokens(&self) -> UnorderedSetMapper<TokenIdentifier>;

    #[storage_mapper("accumulatedRewards")]
    fn accumulated_rewards(&self, token_id: &TokenIdentifier) -> SingleValueMapper<BigUint>;

    #[storage_mapper("rewardCapacity")]
    fn reward_capacity(&self, token_id: &TokenIdentifier) -> SingleValueMapper<BigUint>;
}
