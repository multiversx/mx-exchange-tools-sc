multiversx_sc::imports!();

use common_structs::PaymentsVec;
use farm_staking_proxy::result_types::ClaimDualYieldResult;
use locked_token_wrapper::wrapped_token;

use crate::common::{
    errors::{
        ERROR_BAD_PAYMENT_TOKENS, ERROR_EXTERNAL_CONTRACT_OUTPUT, ERROR_METASTAKING_DOES_NOT_EXIST,
    },
    structs::{MetastakingState, UnstakeMetastakingAttributes, WrappedMetastakingTokenAttributes},
};

#[multiversx_sc::module]
pub trait MetastakingInteractionsModule:
    read_external_storage::ReadExternalStorageModule
    + crate::external_sc_interactions::metastaking_actions::MetastakingActionsModule
    + crate::external_sc_interactions::energy_dao_config::EnergyDAOConfigModule
    + crate::external_sc_interactions::locked_token_actions::LockedTokenModule
    + crate::external_sc_interactions::farm_actions::FarmActionsModule
    + utils::UtilsModule
    + permissions_module::PermissionsModule
    + energy_query::EnergyQueryModule
    + lkmex_transfer::energy_transfer::EnergyTransferModule
    + legacy_token_decode_module::LegacyTokenDecodeModule
    + wrapped_token::WrappedTokenModule
    + simple_lock::token_attributes::TokenAttributesModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    #[payable("*")]
    #[endpoint(enterMetastaking)]
    fn enter_metastaking_endpoint(&self, metastaking_address: ManagedAddress) -> EsdtTokenPayment {
        let payment = self.call_value().single_esdt();
        let mut metastaking_state_mapper = self.metastaking_state(&metastaking_address);
        require!(
            !metastaking_state_mapper.is_empty(),
            ERROR_METASTAKING_DOES_NOT_EXIST
        );

        // We need to first claim the metastaking rewards, as we need to call the enter farm endpoint, which also claims the boosted rewards, if any
        // This way, if there are any pending rewards, they are claimed using the correct Metastaking position, and not during the enter farm flow
        let metastaking_state = metastaking_state_mapper.get();
        if metastaking_state.dual_yield_amount > 0 {
            let empty_payment =
                EsdtTokenPayment::new(payment.token_identifier.clone(), 0u64, BigUint::zero());
            let _ = self.claim_and_compute_user_metastaking_rewards(
                &empty_payment,
                metastaking_address.clone(),
                &mut metastaking_state_mapper,
            );
        }

        let farm_address = self.get_lp_farm_address(metastaking_address.clone());
        let farming_token_id = self.get_farming_token(farm_address.clone());
        require!(
            farming_token_id == payment.token_identifier,
            ERROR_BAD_PAYMENT_TOKENS
        );

        // We need to enter the farm for the user, in order to have the Energy DAO contract as the original_owner of the position
        // This is needed later on, in order to be able to merge farm positions when adding extra position
        let enter_farm_payment = ManagedVec::from_single_item(payment.clone());
        let enter_farm_result = self.call_enter_farm(farm_address, enter_farm_payment);
        let (new_farm_token, _) = enter_farm_result.into_tuple();

        // Proceed to enter metastaking with the new farm position
        let metastaking_state = metastaking_state_mapper.get();
        let dual_yield_token_id = self.get_dual_yield_token(metastaking_address.clone());
        let farm_staking_address = self.get_staking_farm_address(metastaking_address.clone());
        let division_safety_constant =
            self.get_division_safety_constant(farm_staking_address.clone());
        let mut enter_metastaking_payments = ManagedVec::from_single_item(new_farm_token.clone());

        let current_metastaking_position = EsdtTokenPayment::new(
            dual_yield_token_id,
            metastaking_state.dual_yield_token_nonce,
            metastaking_state.dual_yield_amount.clone(),
        );

        if current_metastaking_position.amount > 0 {
            enter_metastaking_payments.push(current_metastaking_position);
        }

        let enter_metastaking_result =
            self.call_enter_metastaking(metastaking_address.clone(), enter_metastaking_payments);

        require!(
            enter_metastaking_result.dual_yield_tokens.amount > metastaking_state.dual_yield_amount,
            ERROR_EXTERNAL_CONTRACT_OUTPUT
        );

        self.update_metastaking_after_claim(
            &metastaking_state,
            &mut metastaking_state_mapper,
            &new_farm_token.amount,
            &enter_metastaking_result.dual_yield_tokens,
            enter_metastaking_result.lp_farm_boosted_rewards,
            enter_metastaking_result.staking_boosted_rewards, // they are given in reward tokens, not in locked lp farm tokens
            &division_safety_constant,
        );

        let caller = self.blockchain().get_caller();
        let new_metastaking_state = metastaking_state_mapper.get();
        let user_token_attributes = WrappedMetastakingTokenAttributes {
            metastaking_address,
            lp_farm_token_rps: new_metastaking_state.lp_farm_rps,
            staking_token_rps: new_metastaking_state.staking_rps,
        };

        self.wrapped_metastaking_token().nft_create_and_send(
            &caller,
            new_farm_token.amount,
            &user_token_attributes,
        )
    }

    #[payable("*")]
    #[endpoint(unstakeMetastaking)]
    fn unstake_metastaking(&self) -> PaymentsVec<Self::Api> {
        let payment = self.call_value().single_esdt();
        require!(
            payment.token_identifier == self.wrapped_metastaking_token().get_token_id(),
            ERROR_BAD_PAYMENT_TOKENS
        );
        let token_attributes: WrappedMetastakingTokenAttributes<Self::Api> =
            self.get_token_attributes(&payment.token_identifier, payment.token_nonce);
        let metastaking_address = token_attributes.metastaking_address;
        let mut metastaking_state_mapper = self.metastaking_state(&metastaking_address);
        require!(
            !metastaking_state_mapper.is_empty(),
            ERROR_METASTAKING_DOES_NOT_EXIST
        );

        let metastaking_state = metastaking_state_mapper.get();
        let dual_yield_token_id = self.get_dual_yield_token(metastaking_address.clone());
        let farm_staking_address = self.get_staking_farm_address(metastaking_address.clone());
        let division_safety_constant =
            self.get_division_safety_constant(farm_staking_address.clone());
        let _full_dual_yield_position = EsdtTokenPayment::new(
            dual_yield_token_id.clone(),
            metastaking_state.dual_yield_token_nonce,
            metastaking_state.dual_yield_amount.clone(),
        );

        // As the dual yield amount changes every time rewards are claimed
        // We use the fixed amount of farm tokens that the user initially provided to keep track of his position
        // To compute the correct unstake amount, we apply the rule of three to the amount of tokens that the user sent as payment
        let unstake_amount = (&payment.amount * &metastaking_state.dual_yield_amount)
            / &metastaking_state.metastaking_token_supply;

        let unstake_dual_yield_tokens = EsdtTokenPayment::new(
            dual_yield_token_id.clone(),
            metastaking_state.dual_yield_token_nonce,
            unstake_amount.clone(),
        );

        let unstake_result =
            self.call_exit_metastaking(metastaking_address.clone(), unstake_dual_yield_tokens);

        let update_dual_yield_tokens = EsdtTokenPayment::new(
            dual_yield_token_id,
            metastaking_state.dual_yield_token_nonce,
            metastaking_state.dual_yield_amount.clone() - unstake_amount,
        );

        self.update_metastaking_after_claim(
            &metastaking_state,
            &mut metastaking_state_mapper,
            &BigUint::zero(),
            &update_dual_yield_tokens,
            unstake_result.lp_farm_rewards.clone(),
            unstake_result.staking_rewards.clone(),
            &division_safety_constant,
        );

        let (user_lp_farm_reward, user_staking_reward) = self.compute_user_metastaking_rewards(
            &mut metastaking_state_mapper,
            &payment,
            &division_safety_constant,
        );

        metastaking_state_mapper.update(|config| {
            config.lp_farm_reward_reserve -= &user_lp_farm_reward;
            config.staking_reward_reserve -= &user_staking_reward;

            // We update the token supply after the rewards are calculated, to also include the current user in the computation
            config.metastaking_token_supply -= &payment.amount;
        });

        self.send().esdt_local_burn(
            &payment.token_identifier,
            payment.token_nonce,
            &payment.amount,
        );

        let new_metastaking_state = metastaking_state_mapper.get();
        let unstake_attributes = UnstakeMetastakingAttributes {
            metastaking_address,
            unbond_token_id: unstake_result.unbond_staking_farm_token.token_identifier,
            unbond_token_nonce: unstake_result.unbond_staking_farm_token.token_nonce,
        };
        let unbond_token_payment = self.unstake_metastaking_token().nft_create(
            unstake_result.unbond_staking_farm_token.amount,
            &unstake_attributes,
        );

        let mut user_payments = ManagedVec::from_single_item(unbond_token_payment);
        if user_lp_farm_reward > 0 {
            let user_lp_farm_reward_payment = EsdtTokenPayment::new(
                unstake_result.lp_farm_rewards.token_identifier,
                new_metastaking_state.lp_farm_reward_token_nonce,
                user_lp_farm_reward,
            );
            let wrapper_user_lp_farm_rewards = self.wrap_locked_token(user_lp_farm_reward_payment);
            user_payments.push(wrapper_user_lp_farm_rewards);
        }
        if user_staking_reward > 0 {
            let user_staking_reward_payment = EsdtTokenPayment::new(
                unstake_result.staking_rewards.token_identifier,
                0u64,
                user_staking_reward,
            );
            user_payments.push(user_staking_reward_payment);
        }
        if unstake_result.other_token_payment.amount > 0 {
            let mut user_other_token_payment = unstake_result.other_token_payment;
            self.apply_fee(&mut user_other_token_payment);
            user_payments.push(user_other_token_payment);
        }

        let caller = self.blockchain().get_caller();
        self.send().direct_multi(&caller, &user_payments);

        user_payments
    }

    #[payable("*")]
    #[endpoint(unbondMetastaking)]
    fn unbond_metastaking_endpoint(&self) -> EsdtTokenPayment<Self::Api> {
        let payment = self.call_value().single_esdt();
        require!(
            payment.token_identifier == self.unstake_metastaking_token().get_token_id(),
            ERROR_BAD_PAYMENT_TOKENS
        );
        let token_attributes: UnstakeMetastakingAttributes<Self::Api> =
            self.get_token_attributes(&payment.token_identifier, payment.token_nonce);
        let metastaking_address = token_attributes.metastaking_address;
        let metastaking_state_mapper = self.metastaking_state(&metastaking_address);
        require!(
            !metastaking_state_mapper.is_empty(),
            ERROR_METASTAKING_DOES_NOT_EXIST
        );

        let unbond_payment = EsdtTokenPayment::new(
            token_attributes.unbond_token_id,
            token_attributes.unbond_token_nonce,
            payment.amount.clone(),
        );
        let mut unbond_payment = self.call_unbond_metastaking(metastaking_address, unbond_payment);

        self.send().esdt_local_burn(
            &payment.token_identifier,
            payment.token_nonce,
            &payment.amount,
        );
        self.apply_fee(&mut unbond_payment);

        let caller = self.blockchain().get_caller();
        self.send()
            .direct_non_zero_esdt_payment(&caller, &unbond_payment);

        unbond_payment
    }

    #[endpoint(claimMetastakingRewards)]
    fn claim_metastaking_rewards(&self) -> PaymentsVec<Self::Api> {
        let payment = self.call_value().single_esdt();
        require!(
            payment.token_identifier == self.wrapped_metastaking_token().get_token_id(),
            ERROR_BAD_PAYMENT_TOKENS
        );
        let token_attributes: WrappedMetastakingTokenAttributes<Self::Api> =
            self.get_token_attributes(&payment.token_identifier, payment.token_nonce);
        let metastaking_address = token_attributes.metastaking_address;
        let mut metastaking_state_mapper = self.metastaking_state(&metastaking_address);
        require!(
            !metastaking_state_mapper.is_empty(),
            ERROR_METASTAKING_DOES_NOT_EXIST
        );

        let claim_result = self.claim_and_compute_user_metastaking_rewards(
            &payment,
            metastaking_address.clone(),
            &mut metastaking_state_mapper,
        );

        let new_metastaking_state = metastaking_state_mapper.get();
        let new_attributes = WrappedMetastakingTokenAttributes {
            metastaking_address,
            lp_farm_token_rps: new_metastaking_state.lp_farm_rps,
            staking_token_rps: new_metastaking_state.staking_rps,
        };
        let new_metastaking_token = self
            .wrapped_metastaking_token()
            .nft_create(payment.amount.clone(), &new_attributes);
        let mut user_payments = ManagedVec::from_single_item(new_metastaking_token);
        if claim_result.lp_farm_rewards.amount > 0 {
            let wrapper_lp_farm_rewards = self.wrap_locked_token(claim_result.lp_farm_rewards);
            user_payments.push(wrapper_lp_farm_rewards);
        }
        if claim_result.staking_farm_rewards.amount > 0 {
            user_payments.push(claim_result.staking_farm_rewards);
        }
        let caller = self.blockchain().get_caller();
        self.send().direct_multi(&caller, &user_payments);

        user_payments
    }

    fn claim_and_compute_user_metastaking_rewards(
        &self,
        payment: &EsdtTokenPayment<Self::Api>,
        metastaking_address: ManagedAddress,
        metastaking_state_mapper: &mut SingleValueMapper<MetastakingState<Self::Api>>,
    ) -> ClaimDualYieldResult<Self::Api> {
        let metastaking_state = metastaking_state_mapper.get();
        let dual_yield_token_id = self.get_dual_yield_token(metastaking_address.clone());
        let farm_staking_address = self.get_staking_farm_address(metastaking_address.clone());
        let division_safety_constant = self.get_division_safety_constant(farm_staking_address);
        let current_metastaking_position = EsdtTokenPayment::new(
            dual_yield_token_id,
            metastaking_state.dual_yield_token_nonce,
            metastaking_state.dual_yield_amount.clone(),
        );
        let claim_rewards_result =
            self.call_metastaking_claim(metastaking_address.clone(), current_metastaking_position);

        self.update_metastaking_after_claim(
            &metastaking_state,
            metastaking_state_mapper,
            &BigUint::zero(),
            &claim_rewards_result.new_dual_yield_tokens,
            claim_rewards_result.lp_farm_rewards,
            claim_rewards_result.staking_farm_rewards,
            &division_safety_constant,
        );

        if payment.amount > 0 {
            self.send().esdt_local_burn(
                &payment.token_identifier,
                payment.token_nonce,
                &payment.amount,
            );
        }

        let (user_lp_farm_reward, user_staking_reward) = self.compute_user_metastaking_rewards(
            metastaking_state_mapper,
            payment,
            &division_safety_constant,
        );

        metastaking_state_mapper.update(|config| {
            config.lp_farm_reward_reserve -= &user_lp_farm_reward;
            config.staking_reward_reserve -= &user_staking_reward;
        });

        let new_metastaking_state = metastaking_state_mapper.get();
        let locked_token_id = self.get_locked_token_id();
        let user_lp_farm_reward_payment = EsdtTokenPayment::new(
            locked_token_id,
            new_metastaking_state.lp_farm_reward_token_nonce,
            user_lp_farm_reward,
        );
        let staking_token_id = self.get_staking_token(metastaking_address);
        let user_staking_reward_payment =
            EsdtTokenPayment::new(staking_token_id, 0u64, user_staking_reward);

        ClaimDualYieldResult {
            lp_farm_rewards: user_lp_farm_reward_payment,
            staking_farm_rewards: user_staking_reward_payment,
            new_dual_yield_tokens: claim_rewards_result.new_dual_yield_tokens,
        }
    }

    #[proxy]
    fn metastaking_proxy(&self, sc_address: ManagedAddress)
        -> farm_staking_proxy::Proxy<Self::Api>;
}
