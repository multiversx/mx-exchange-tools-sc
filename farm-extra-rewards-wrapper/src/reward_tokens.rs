use common_structs::{Nonce, PaymentsVec};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait RewardTokensModule: permissions_module::PermissionsModule {
    /// Deposit tokens and add them to the whitelist
    #[payable("*")]
    #[endpoint(depositRewardTokens)]
    fn deposit_reward_tokens(&self) {
        self.require_caller_has_owner_or_admin_permissions();

        let payments = self.call_value().all_esdt_transfers();
        let current_block = self.blockchain().get_block_nonce();
        let mut tokens_mapper = self.reward_tokens();
        for payment in &payments {
            self.reward_capacity(&payment.token_identifier)
                .update(|total| *total += &payment.amount);

            let is_new = tokens_mapper.insert(payment.token_identifier.clone());
            if is_new {
                self.token_addition_block(&payment.token_identifier)
                    .set(current_block);
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

        let mut output_payments = PaymentsVec::new();
        let mut tokens_mapper = self.reward_tokens();
        for token in tokens {
            let _ = tokens_mapper.swap_remove(&token);
            self.token_addition_block(&token).clear();

            let accumulated_rewards = self.accumulated_rewards(&token).take();
            let capacity = self.reward_capacity(&token).take();
            let remaining_tokens = capacity - accumulated_rewards;
            if remaining_tokens == 0 {
                continue;
            }

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
    #[storage_mapper("rewTokens")]
    fn reward_tokens(&self) -> UnorderedSetMapper<TokenIdentifier>;

    #[view(getTokenAdditionBlock)]
    #[storage_mapper("tokenAddBlk")]
    fn token_addition_block(&self, token_id: &TokenIdentifier) -> SingleValueMapper<Nonce>;

    #[storage_mapper("accRew")]
    fn accumulated_rewards(&self, token_id: &TokenIdentifier) -> SingleValueMapper<BigUint>;

    #[storage_mapper("rewCap")]
    fn reward_capacity(&self, token_id: &TokenIdentifier) -> SingleValueMapper<BigUint>;
}
