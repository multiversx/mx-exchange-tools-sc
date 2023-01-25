use common_structs::PaymentsVec;

elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait WithdrawTokensModule {
    fn withdraw_specific_tokens(
        &self,
        user: &ManagedAddress,
        tokens_mapper: &SingleValueMapper<PaymentsVec<Self::Api>>,
        tokens_to_withdraw: &PaymentsVec<Self::Api>,
    ) {
        if tokens_to_withdraw.is_empty() {
            return;
        }

        let mut all_tokens = tokens_mapper.get();
        for ttw in tokens_to_withdraw {
            let opt_index = self.find_token_in_payments(&ttw.token_identifier, &all_tokens);
            require!(opt_index.is_some(), "Invalid token to withdraw");

            let index = unsafe { opt_index.unwrap_unchecked() };
            let mut full_token = all_tokens.get(index);
            require!(
                ttw.amount <= full_token.amount,
                "Not enough balance to withdraw"
            );

            full_token.amount -= ttw.amount;
            if full_token.amount > 0 {
                let _ = all_tokens.set(index, &full_token);
            } else {
                all_tokens.remove(index);
            }
        }

        tokens_mapper.set(&all_tokens);

        self.send().direct_multi(user, tokens_to_withdraw);
    }

    fn withdraw_all_tokens(
        &self,
        user: &ManagedAddress,
        tokens_mapper: &SingleValueMapper<PaymentsVec<Self::Api>>,
    ) -> PaymentsVec<Self::Api> {
        let tokens = tokens_mapper.take();
        if !tokens.is_empty() {
            self.send().direct_multi(user, &tokens);
        }

        tokens
    }

    /// Returns `Some(index)` at which it is located if found, `None` otherwise
    fn find_token_in_payments(
        &self,
        token_id: &TokenIdentifier,
        payments: &PaymentsVec<Self::Api>,
    ) -> Option<usize> {
        for (i, payment) in payments.iter().enumerate() {
            if &payment.token_identifier == token_id {
                return Some(i);
            }
        }

        None
    }
}
