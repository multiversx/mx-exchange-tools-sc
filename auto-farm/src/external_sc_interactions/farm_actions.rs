use common_structs::PaymentsVec;
use farm::{
    base_functions::{ClaimRewardsResultType, ClaimRewardsResultWrapper},
    EnterFarmResultType,
};
use farm_staking::stake_farm::ProxyTrait as _;

use crate::{
    common::{
        address_to_id_mapper::{AddressId, NULL_ID},
        rewards_wrapper::RewardsWrapper,
    },
    external_storage_read::farm_storage_read::State,
};

elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait FarmActionsModule:
    crate::common::common_storage::CommonStorageModule
    + crate::whitelists::farms_whitelist::FarmsWhitelistModule
    + crate::external_storage_read::farm_storage_read::FarmStorageReadModule
    + crate::user_tokens::user_farm_tokens::UserFarmTokensModule
    + crate::user_tokens::user_rewards::UserRewardsModule
    + crate::fees::FeesModule
    + utils::UtilsModule
    + energy_query::EnergyQueryModule
    + crate::external_sc_interactions::locked_token_merging::LockedTokenMergingModule
    + lkmex_transfer::energy_transfer::EnergyTransferModule
    + legacy_token_decode_module::LegacyTokenDecodeModule
{
    fn claim_all_farm_rewards(
        &self,
        user: &ManagedAddress,
        user_id: AddressId,
        rew_wrapper: &mut RewardsWrapper<Self::Api>,
    ) {
        let farms_mapper = self.farm_ids();
        let user_tokens_mapper = self.user_farm_tokens(user_id);
        let user_farm_tokens = user_tokens_mapper.get();
        if user_farm_tokens.is_empty() {
            return;
        }

        let mut new_user_farm_tokens = PaymentsVec::new();
        for farm_token in &user_farm_tokens {
            let farm_id = self.farm_for_farm_token(&farm_token.token_identifier).get();
            let opt_farm_addr = farms_mapper.get_address(farm_id);
            if opt_farm_addr.is_none() {
                new_user_farm_tokens.push(farm_token);
                continue;
            }

            let farm_addr = unsafe { opt_farm_addr.unwrap_unchecked() };
            let farm_state = self.get_farm_state(&farm_addr);
            if farm_state != State::Active {
                new_user_farm_tokens.push(farm_token);
                continue;
            }

            let claim_result = self.call_farm_claim(farm_addr, user.clone(), farm_token);
            new_user_farm_tokens.push(claim_result.new_farm_token);

            rew_wrapper.add_tokens(claim_result.rewards);
        }

        user_tokens_mapper.set(&new_user_farm_tokens);
    }

    /// user_farm_ids contains the associated farm_id for each token in user_farm_tokens
    fn compound_staking_rewards_with_existing_farm_position(
        &self,
        user: &ManagedAddress,
        user_farm_tokens: &mut PaymentsVec<Self::Api>,
        user_farm_ids: &ManagedVec<AddressId>,
        new_tokens: EsdtTokenPayment,
    ) -> Result<(), ()> {
        let farm_id = self
            .farm_for_farming_token(&new_tokens.token_identifier)
            .get();
        if farm_id == NULL_ID {
            return Result::Err(());
        }

        let opt_existing_farm_index = user_farm_ids.find(&farm_id);
        if opt_existing_farm_index.is_none() {
            return Result::Err(());
        }

        let existing_farm_index = unsafe { opt_existing_farm_index.unwrap_unchecked() };
        let opt_farm_addr = self
            .farm_ids()
            .get_address(existing_farm_index as AddressId);
        if opt_farm_addr.is_none() {
            return Result::Err(());
        }

        let farm_addr = unsafe { opt_farm_addr.unwrap_unchecked() };
        let existing_farm_pos = user_farm_tokens.get(existing_farm_index);
        let new_farm_token = self.call_enter_farm_staking_with_additional_tokens(
            farm_addr,
            user.clone(),
            existing_farm_pos,
            new_tokens,
        );
        let _ = user_farm_tokens.set(existing_farm_index, &new_farm_token);

        Result::Ok(())
    }

    fn call_enter_farm_staking_with_additional_tokens(
        &self,
        farm_addr: ManagedAddress,
        user: ManagedAddress,
        farm_token: EsdtTokenPayment,
        new_tokens: EsdtTokenPayment,
    ) -> EsdtTokenPayment {
        let raw_results: EnterFarmResultType<Self::Api> = self
            .farm_staking_proxy(farm_addr)
            .stake_farm_endpoint(user)
            .with_esdt_transfer(new_tokens)
            .with_esdt_transfer(farm_token)
            .execute_on_dest_context();

        // since we already claimed, there are no boosted rewards
        let (new_farm_token, _) = raw_results.into_tuple();
        new_farm_token
    }

    fn call_farm_claim(
        &self,
        farm_addr: ManagedAddress,
        user: ManagedAddress,
        farm_token: EsdtTokenPayment,
    ) -> ClaimRewardsResultWrapper<Self::Api> {
        let raw_results: ClaimRewardsResultType<Self::Api> = self
            .farm_proxy(farm_addr)
            .claim_rewards_endpoint(user)
            .with_esdt_transfer(farm_token)
            .execute_on_dest_context();
        let (new_farm_token, rewards) = raw_results.into_tuple();

        ClaimRewardsResultWrapper {
            new_farm_token,
            rewards,
        }
    }

    #[proxy]
    fn farm_proxy(&self, sc_address: ManagedAddress) -> farm_with_locked_rewards::Proxy<Self::Api>;

    #[proxy]
    fn farm_staking_proxy(&self, sc_address: ManagedAddress) -> farm_staking::Proxy<Self::Api>;
}
