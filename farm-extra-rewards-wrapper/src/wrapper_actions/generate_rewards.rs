use common_structs::{PaymentAttributesPair, PaymentsVec};
use contexts::{claim_rewards_context::ClaimRewardsContext, storage_cache::StorageCache};
use farm_base_impl::base_traits_impl::FarmContract;
use fixed_supply_token::FixedSupplyToken;

use crate::{
    common::payments_wrapper::PaymentsWrapper,
    single_token_rewards::{BaseFarmLogicWrapper, ScTraits},
    wrapped_farm_attributes::WrappedFarmAttributes,
};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub struct InternalClaimResult<'a, C>
where
    C: ScTraits,
{
    pub rewards: PaymentsWrapper<C::Api>,
    pub underlying_farm_tokens: PaymentsWrapper<C::Api>,
    pub claim_context: ClaimRewardsContext<C::Api, WrappedFarmAttributes<C::Api>>,
    pub storage_cache: StorageCache<'a, C>,
}

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Debug)]
pub struct ClaimResult<M: ManagedTypeApi> {
    pub new_wrapped_farm_token: EsdtTokenPayment<M>,
    pub rewards: PaymentsWrapper<M>,
}

#[multiversx_sc::module]
pub trait GenerateRewardsModule:
    read_external_storage::ReadExternalStorageModule
    + auto_farm::whitelists::farms_whitelist::FarmsWhitelistModule
    + auto_farm::external_storage_read::farm_storage_read::FarmStorageReadModule
    + rewards::RewardsModule
    + config::ConfigModule
    + token_send::TokenSendModule
    + farm_token::FarmTokenModule
    + pausable::PausableModule
    + permissions_module::PermissionsModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
    + farm_base_impl::base_farm_validation::BaseFarmValidationModule
    + farm_base_impl::enter_farm::BaseEnterFarmModule
    + utils::UtilsModule
    + crate::reward_tokens::RewardTokensModule
    + crate::external_sc_interactions::farm_interactions::FarmInteractionsModule
{
    #[payable("*")]
    #[endpoint(claimRewards)]
    fn claim_rewards(&self) -> ClaimResult<Self::Api> {
        let caller = self.blockchain().get_caller();
        let payments = self.get_non_empty_payments();
        let mut claim_result = self.generate_rewards_all_tokens(&caller, payments);

        let farm_claim_result = self.claim_base_farm_rewards(
            caller.clone(),
            claim_result.underlying_farm_tokens.into_payments(),
        );
        claim_result.rewards.push(farm_claim_result.rewards);

        let new_wrapped_farm_token = self
            .create_new_wrapped_farm_token_after_claim(
                caller.clone(),
                claim_result.claim_context,
                farm_claim_result.new_farm_token,
                &claim_result.storage_cache,
            )
            .payment;
        self.send()
            .direct_non_zero_esdt_payment(&caller, &new_wrapped_farm_token);

        claim_result.rewards.send_to(&caller);

        ClaimResult {
            new_wrapped_farm_token,
            rewards: claim_result.rewards,
        }
    }

    fn generate_rewards_all_tokens(
        &self,
        caller: &ManagedAddress,
        payments: PaymentsVec<Self::Api>,
    ) -> InternalClaimResult<Self> {
        let mut storage_cache = StorageCache::new(self);
        self.validate_contract_state(storage_cache.contract_state, &storage_cache.farm_token_id);

        let claim_rewards_context = ClaimRewardsContext::<
            Self::Api,
            WrappedFarmAttributes<Self::Api>,
        >::new(
            payments, &storage_cache.farm_token_id, self.blockchain()
        );
        let underlying_farm_tokens = self.get_all_underlying_farm_tokens(&claim_rewards_context);

        let wrapped_farm_token_amount = &claim_rewards_context.first_farm_token.payment.amount;
        let wrapped_token_attributes = claim_rewards_context
            .first_farm_token
            .attributes
            .clone()
            .into_part(wrapped_farm_token_amount);

        let rps_before = storage_cache.reward_per_share.clone();
        let mut max_new_rps = rps_before.clone();
        let mut rewards = PaymentsWrapper::new();
        for token in self.reward_tokens().iter() {
            storage_cache.reward_token_id = token.clone();
            storage_cache.reward_per_share = rps_before.clone();
            BaseFarmLogicWrapper::generate_aggregated_rewards(self, &mut storage_cache);

            max_new_rps = core::cmp::max(max_new_rps, storage_cache.reward_per_share.clone());

            let rew = self.generate_single_token_reward(
                caller,
                token,
                wrapped_farm_token_amount,
                &wrapped_token_attributes,
                &mut storage_cache,
            );
            rewards.push(rew);
        }

        storage_cache.reward_per_share = max_new_rps;

        let block_nonce = self.blockchain().get_block_nonce();
        self.last_reward_block_nonce().set(block_nonce);

        InternalClaimResult {
            rewards,
            underlying_farm_tokens,
            claim_context: claim_rewards_context,
            storage_cache,
        }
    }

    fn generate_single_token_reward(
        &self,
        caller: &ManagedAddress,
        reward_token_id: TokenIdentifier,
        wrapped_farm_amount: &BigUint,
        wrapped_token_attributes: &WrappedFarmAttributes<Self::Api>,
        storage_cache: &mut StorageCache<Self>,
    ) -> EsdtTokenPayment {
        let token_addition_block = self.token_addition_block(&reward_token_id).get();
        if wrapped_token_attributes.creation_block < token_addition_block {
            return EsdtTokenPayment::new(reward_token_id, 0, BigUint::zero());
        }

        let rew_amount = BaseFarmLogicWrapper::calculate_rewards(
            self,
            caller,
            wrapped_farm_amount,
            wrapped_token_attributes,
            storage_cache,
        );
        EsdtTokenPayment::new(reward_token_id, 0, rew_amount)
    }

    fn create_new_wrapped_farm_token_after_claim(
        &self,
        caller: ManagedAddress,
        claim_rewards_context: ClaimRewardsContext<Self::Api, WrappedFarmAttributes<Self::Api>>,
        new_farm_token: EsdtTokenPayment,
        storage_cache: &StorageCache<Self>,
    ) -> PaymentAttributesPair<Self::Api, WrappedFarmAttributes<Self::Api>> {
        let farm_token_mapper = self.farm_token();
        let farm_token_attr = claim_rewards_context
            .first_farm_token
            .attributes
            .into_part(&claim_rewards_context.first_farm_token.payment.amount);
        let base_attributes = BaseFarmLogicWrapper::create_claim_rewards_initial_attributes(
            self,
            caller,
            farm_token_attr,
            storage_cache.reward_per_share.clone(),
        );
        let mut new_token_attributes = self.merge_attributes_from_payments(
            base_attributes,
            &claim_rewards_context.additional_payments,
            &farm_token_mapper,
        );
        new_token_attributes.farm_token_id = new_farm_token.token_identifier;
        new_token_attributes.farm_token_nonce = new_farm_token.token_nonce;

        let new_wrapped_token = farm_token_mapper.nft_create(
            new_token_attributes.get_total_supply(),
            &new_token_attributes,
        );
        let payment_attr_pair = PaymentAttributesPair {
            payment: new_wrapped_token,
            attributes: new_token_attributes,
        };

        let first_farm_token = &claim_rewards_context.first_farm_token.payment;
        farm_token_mapper.nft_burn(first_farm_token.token_nonce, &first_farm_token.amount);
        self.send()
            .esdt_local_burn_multi(&claim_rewards_context.additional_payments);

        payment_attr_pair
    }

    fn get_all_underlying_farm_tokens(
        &self,
        claim_rewards_context: &ClaimRewardsContext<Self::Api, WrappedFarmAttributes<Self::Api>>,
    ) -> PaymentsWrapper<Self::Api> {
        let wrapped_token_mapper = self.farm_token();
        let first_farm_token = &claim_rewards_context
            .first_farm_token
            .attributes
            .farm_token_id;

        let mut underlying_farm_tokens = PaymentsWrapper::new();
        let first_attributes: WrappedFarmAttributes<Self::Api> = claim_rewards_context
            .first_farm_token
            .attributes
            .clone()
            .into_part(&claim_rewards_context.first_farm_token.payment.amount);

        let first_payment = EsdtTokenPayment::new(
            first_farm_token.clone(),
            first_attributes.farm_token_nonce,
            first_attributes.current_token_amount,
        );
        underlying_farm_tokens.push(first_payment);

        for other_wrapped_token in &claim_rewards_context.additional_payments {
            let attributes: WrappedFarmAttributes<Self::Api> = self
                .get_attributes_as_part_of_fixed_supply(
                    &other_wrapped_token,
                    &wrapped_token_mapper,
                );
            require!(
                first_farm_token == &attributes.farm_token_id,
                "Invalid payments, all wrapped tokens must belong to the same farm"
            );

            let payment = EsdtTokenPayment::new(
                attributes.farm_token_id,
                attributes.farm_token_nonce,
                attributes.current_token_amount,
            );
            underlying_farm_tokens.push(payment);
        }

        underlying_farm_tokens
    }
}
