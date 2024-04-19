multiversx_sc::imports!();

use crate::events::{DepositType, WithdrawType};
use common_structs::PaymentsVec;

#[multiversx_sc::module]
pub trait UserMetastakingTokensModule:
    read_external_storage::ReadExternalStorageModule
    + crate::common::common_storage::CommonStorageModule
    + crate::whitelists::metastaking_whitelist::MetastakingWhitelistModule
    + crate::external_storage_read::metastaking_storage_read::MetastakingStorageReadModule
    + super::withdraw_tokens::WithdrawTokensModule
    + crate::events::EventsModule
    + utils::UtilsModule
{
    #[payable("*")]
    #[endpoint(depositMetastakingTokens)]
    fn deposit_metastaking_tokens(&self) {
        let payments = self.get_non_empty_payments();
        let caller = self.blockchain().get_caller();
        let user_id = self.user_ids().get_id_or_insert(&caller);

        self.user_metastaking_tokens(user_id).update(|tokens| {
            for payment in &payments {
                let ms_id = self
                    .metastaking_for_dual_yield_token(&payment.token_identifier)
                    .get();
                require!(ms_id != NULL_ID, "Invalid token");

                tokens.push(payment);
            }
        });

        self.emit_token_deposit_event(&caller, DepositType::MetastakingTokens, &payments);
    }

    #[endpoint(withdrawAllMetastakingTokens)]
    fn withdraw_all_metastaking_tokens_endpoint(&self) -> PaymentsVec<Self::Api> {
        let caller = self.blockchain().get_caller();
        let user_id = self.user_ids().get_id_non_zero(&caller);
        let tokens_mapper = self.user_metastaking_tokens(user_id);
        let withdrawn_tokens = self.withdraw_all_tokens(&caller, &tokens_mapper);
        self.emit_token_withdrawal_event(
            &caller,
            WithdrawType::MetastakingTokens,
            &withdrawn_tokens,
        );

        withdrawn_tokens
    }

    #[endpoint(withdrawSpecificMetastakingTokens)]
    fn withdraw_specific_metastaking_tokens_endpoint(
        &self,
        tokens_to_withdraw: PaymentsVec<Self::Api>,
    ) {
        let caller = self.blockchain().get_caller();
        let user_id = self.user_ids().get_id_non_zero(&caller);
        let tokens_mapper = self.user_metastaking_tokens(user_id);
        self.withdraw_specific_tokens(&caller, &tokens_mapper, &tokens_to_withdraw);
        self.emit_token_withdrawal_event(
            &caller,
            WithdrawType::MetastakingTokens,
            &tokens_to_withdraw,
        );
    }

    #[view(getUserMetastakingTokens)]
    fn get_user_metastaking_tokens_view(&self, user: ManagedAddress) -> PaymentsVec<Self::Api> {
        let user_id = self.user_ids().get_id(&user);
        self.user_metastaking_tokens(user_id).get()
    }

    #[storage_mapper("userMSTokens")]
    fn user_metastaking_tokens(
        &self,
        user_id: AddressId,
    ) -> SingleValueMapper<PaymentsVec<Self::Api>>;
}
