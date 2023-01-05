use common_structs::PaymentsVec;

use crate::common::address_to_id_mapper::{AddressId, NULL_ID};

elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait UserMetastakingTokensModule:
    crate::common::common_storage::CommonStorageModule
    + crate::whitelists::metastaking_whitelist::MetastakingWhitelistModule
    + crate::external_storage_read::metastaking_storage_read::MetastakingStorageReadModule
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
    }

    #[endpoint(withdrawMetastakingTokens)]
    fn withdraw_metastaking_tokens_endpoint(&self) -> PaymentsVec<Self::Api> {
        let caller = self.blockchain().get_caller();
        let user_id = self.user_ids().get_id_non_zero(&caller);
        self.withdraw_metastaking_tokens(&caller, user_id)
    }

    fn withdraw_metastaking_tokens(
        &self,
        user: &ManagedAddress,
        user_id: AddressId,
    ) -> PaymentsVec<Self::Api> {
        let tokens = self.user_metastaking_tokens(user_id).take();
        if !tokens.is_empty() {
            self.send().direct_multi(user, &tokens);
        }

        tokens
    }

    #[view(getUserMetastakingTokens)]
    fn get_user_metastaking_tokens_view(&self, user: ManagedAddress) -> PaymentsVec<Self::Api> {
        let user_id = self.user_ids().get_id(&user);
        if user_id != NULL_ID {
            self.user_metastaking_tokens(user_id).get()
        } else {
            PaymentsVec::new()
        }
    }

    #[storage_mapper("userMSTokens")]
    fn user_metastaking_tokens(
        &self,
        user_id: AddressId,
    ) -> SingleValueMapper<PaymentsVec<Self::Api>>;
}
