use common_structs::PaymentsVec;

use crate::common::address_to_id_mapper::{AddressId, NULL_ID};

elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait UserFarmTokensModule:
    crate::common::common_storage::CommonStorageModule
    + crate::whitelists::farms_whitelist::FarmsWhitelistModule
    + crate::external_storage_read::farm_storage_read::FarmStorageReadModule
    + super::withdraw_tokens::WithdrawTokensModule
    + utils::UtilsModule
{
    #[payable("*")]
    #[endpoint(depositFarmTokens)]
    fn deposit_farm_tokens(&self) {
        let payments = self.get_non_empty_payments();
        let caller = self.blockchain().get_caller();
        let user_id = self.user_ids().get_id_or_insert(&caller);

        self.user_farm_tokens(user_id).update(|tokens| {
            for payment in &payments {
                let farm_id = self.farm_for_farm_token(&payment.token_identifier).get();
                require!(farm_id != NULL_ID, "Invalid token");

                tokens.push(payment);
            }
        });
    }

    #[endpoint(withdrawAllFarmTokens)]
    fn withdraw_all_farm_tokens_endpoint(&self) -> PaymentsVec<Self::Api> {
        let caller = self.blockchain().get_caller();
        let user_id = self.user_ids().get_id_non_zero(&caller);
        let tokens_mapper = self.user_farm_tokens(user_id);
        self.withdraw_all_tokens(&caller, &tokens_mapper)
    }

    #[endpoint(withdrawSpecificFarmTokens)]
    fn withdraw_specific_farm_tokens_endpoint(&self, tokens_to_withdraw: PaymentsVec<Self::Api>) {
        let caller = self.blockchain().get_caller();
        let user_id = self.user_ids().get_id_non_zero(&caller);
        let tokens_mapper = self.user_farm_tokens(user_id);
        self.withdraw_specific_tokens(&caller, &tokens_mapper, &tokens_to_withdraw);
    }

    #[view(getUserFarmTokens)]
    fn get_user_farm_tokens_view(&self, user: ManagedAddress) -> PaymentsVec<Self::Api> {
        let user_id = self.user_ids().get_id(&user);
        if user_id != NULL_ID {
            self.user_farm_tokens(user_id).get()
        } else {
            PaymentsVec::new()
        }
    }

    #[storage_mapper("userFarmTokens")]
    fn user_farm_tokens(&self, user_id: AddressId) -> SingleValueMapper<PaymentsVec<Self::Api>>;
}
