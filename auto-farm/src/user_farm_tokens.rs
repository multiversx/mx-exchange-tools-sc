use common_structs::PaymentsVec;

use crate::address_to_id_mapper::{AddressId, NULL_ID};

elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait UserFarmTokensModule:
    crate::common_storage::CommonStorageModule
    + crate::farms_whitelist::FarmsWhitelistModule
    + crate::farm_external_storage_read::FarmExternalStorageReadModule
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

    #[endpoint(withdrawFarmTokens)]
    fn withdraw_farm_tokens(&self) -> PaymentsVec<Self::Api> {
        let caller = self.blockchain().get_caller();
        let user_id = self.user_ids().get_id_or_insert(&caller);
        let tokens = self.user_farm_tokens(user_id).take();
        if !tokens.is_empty() {
            self.send().direct_multi(&caller, &tokens);
        }

        tokens
    }

    #[view(getUserFarmTokens)]
    fn get_user_farm_tokens_view(&self, user: ManagedAddress) -> PaymentsVec<Self::Api> {
        let user_id = self.user_ids().get_id_or_insert(&user);
        self.user_farm_tokens(user_id).get()
    }

    #[storage_mapper("userFarmTokens")]
    fn user_farm_tokens(&self, user_id: AddressId) -> SingleValueMapper<PaymentsVec<Self::Api>>;
}
