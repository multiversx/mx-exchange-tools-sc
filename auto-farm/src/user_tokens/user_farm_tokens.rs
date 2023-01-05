use common_structs::PaymentsVec;

use crate::common::address_to_id_mapper::{AddressId, NULL_ID};

elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait UserFarmTokensModule:
    crate::common::common_storage::CommonStorageModule
    + crate::whitelists::farms_whitelist::FarmsWhitelistModule
    + crate::external_sc_interactions::farm_external_storage_read::FarmExternalStorageReadModule
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
    fn withdraw_farm_tokens_endpoint(&self) -> PaymentsVec<Self::Api> {
        let caller = self.blockchain().get_caller();
        let user_id = self.user_ids().get_id_non_zero(&caller);
        self.withdraw_farm_tokens(&caller, user_id)
    }

    fn withdraw_farm_tokens(
        &self,
        user: &ManagedAddress,
        user_id: AddressId,
    ) -> PaymentsVec<Self::Api> {
        let tokens = self.user_farm_tokens(user_id).take();
        if !tokens.is_empty() {
            self.send().direct_multi(user, &tokens);
        }

        tokens
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
