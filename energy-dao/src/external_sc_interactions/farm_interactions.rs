multiversx_sc::imports!();

use crate::common::{
    errors::{
        ERROR_BAD_PAYMENT_TOKENS, ERROR_EXTERNAL_CONTRACT_OUTPUT, ERROR_FARM_DOES_NOT_EXIST,
        ERROR_LP_FARM_METASTAKING_ACTIVE, ERROR_UNBOND_TOO_SOON,
    },
    structs::{FarmState, UnstakeFarmAttributes, WrappedFarmTokenAttributes},
};
use common_structs::PaymentsVec;
use locked_token_wrapper::wrapped_token;

pub type ClaimRewardsResultType<BigUint> =
    MultiValue2<EsdtTokenPayment<BigUint>, EsdtTokenPayment<BigUint>>;

#[multiversx_sc::module]
pub trait FarmInteractionsModule:
    read_external_storage::ReadExternalStorageModule
    + crate::external_sc_interactions::energy_dao_config::EnergyDAOConfigModule
    + crate::external_sc_interactions::farm_actions::FarmActionsModule
    + crate::external_sc_interactions::locked_token_actions::LockedTokenModule
    + crate::external_sc_interactions::fees_collector_interactions::FeesCollectorInteractionsModule
    + lkmex_transfer::energy_transfer::EnergyTransferModule
    + legacy_token_decode_module::LegacyTokenDecodeModule
    + energy_query::EnergyQueryModule
    + token_send::TokenSendModule
    + utils::UtilsModule
    + permissions_module::PermissionsModule
    + wrapped_token::WrappedTokenModule
    + simple_lock::token_attributes::TokenAttributesModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    #[payable("*")]
    #[endpoint(enterFarm)]
    fn enter_farm_endpoint(&self, farm_address: ManagedAddress) -> EsdtTokenPayment {
        require!(
            self.lp_farm_metastaking_address(&farm_address).is_empty(),
            ERROR_LP_FARM_METASTAKING_ACTIVE
        );
        let payment = self.call_value().single_esdt();
        let mut farm_state_mapper = self.farm_state(&farm_address);
        require!(!farm_state_mapper.is_empty(), ERROR_FARM_DOES_NOT_EXIST);
        let farming_token_id = self.get_farming_token(farm_address.clone());
        require!(
            farming_token_id == payment.token_identifier,
            ERROR_BAD_PAYMENT_TOKENS
        );

        let farm_token_id = self.get_farm_token(farm_address.clone());
        let division_safety_constant = self.get_division_safety_constant(farm_address.clone());

        // Needed in order to have the most up-to-date farm state, to properly compute the users positions
        self.claim_and_update_farm_state(farm_address.clone(), &mut farm_state_mapper);

        let farm_state = farm_state_mapper.get();
        let mut enter_farm_payments = ManagedVec::from_single_item(payment.clone());

        let current_farm_position = EsdtTokenPayment::new(
            farm_token_id,
            farm_state.farm_token_nonce,
            farm_state.farm_staked_value.clone(),
        );
        let initial_total_farm_amount = current_farm_position.amount.clone();
        if initial_total_farm_amount > 0 {
            enter_farm_payments.push(current_farm_position);
        }

        let enter_farm_result = self.call_enter_farm(farm_address.clone(), enter_farm_payments);
        let (new_farm_token, farm_rewards) = enter_farm_result.into_tuple();

        require!(
            new_farm_token.amount > initial_total_farm_amount,
            ERROR_EXTERNAL_CONTRACT_OUTPUT
        );

        let user_farm_amount = &new_farm_token.amount - &initial_total_farm_amount;
        self.update_farm_after_claim(
            &farm_state,
            &mut farm_state_mapper,
            &new_farm_token,
            farm_rewards,
            &division_safety_constant,
        );

        let caller = self.blockchain().get_caller();
        let new_farm_state = farm_state_mapper.get();
        let user_token_attributes = WrappedFarmTokenAttributes {
            farm_address,
            token_rps: new_farm_state.farm_rps,
        };
        self.wrapped_farm_token().nft_create_and_send(
            &caller,
            user_farm_amount,
            &user_token_attributes,
        )
    }

    #[payable("*")]
    #[endpoint(claimUserRewards)]
    fn claim_user_rewards(&self) -> PaymentsVec<Self::Api> {
        let payment = self.call_value().single_esdt();
        require!(
            payment.token_identifier == self.wrapped_farm_token().get_token_id(),
            ERROR_BAD_PAYMENT_TOKENS
        );
        let token_attributes: WrappedFarmTokenAttributes<Self::Api> =
            self.get_token_attributes(&payment.token_identifier, payment.token_nonce);
        let farm_address = token_attributes.farm_address;

        require!(
            self.lp_farm_metastaking_address(&farm_address).is_empty(),
            ERROR_LP_FARM_METASTAKING_ACTIVE
        );

        let mut farm_state_mapper = self.farm_state(&farm_address);
        require!(!farm_state_mapper.is_empty(), ERROR_FARM_DOES_NOT_EXIST);

        let (_, user_rewards) = self
            .claim_and_compute_user_rewards(&payment, farm_address.clone(), &mut farm_state_mapper)
            .into_tuple();

        let new_farm_state = farm_state_mapper.get();
        let new_attributes = WrappedFarmTokenAttributes {
            farm_address,
            token_rps: new_farm_state.farm_rps,
        };
        let new_farm_token = self
            .wrapped_farm_token()
            .nft_create(payment.amount.clone(), &new_attributes);
        let mut user_payments = ManagedVec::from_single_item(new_farm_token);
        if user_rewards.amount > 0 {
            let wrapper_user_rewards = self.wrap_locked_token(user_rewards);
            user_payments.push(wrapper_user_rewards);
        }
        let caller = self.blockchain().get_caller();
        self.send().direct_multi(&caller, &user_payments);

        user_payments
    }

    #[payable("*")]
    #[endpoint(unstakeFarm)]
    fn unstake_farm(&self) -> PaymentsVec<Self::Api> {
        let payment = self.call_value().single_esdt();
        require!(
            payment.token_identifier == self.wrapped_farm_token().get_token_id(),
            ERROR_BAD_PAYMENT_TOKENS
        );
        let token_attributes: WrappedFarmTokenAttributes<Self::Api> =
            self.get_token_attributes(&payment.token_identifier, payment.token_nonce);
        let farm_address = token_attributes.farm_address;

        require!(
            self.lp_farm_metastaking_address(&farm_address).is_empty(),
            ERROR_LP_FARM_METASTAKING_ACTIVE
        );

        let mut farm_state_mapper = self.farm_state(&farm_address);
        require!(!farm_state_mapper.is_empty(), ERROR_FARM_DOES_NOT_EXIST);

        let (new_farm_token, user_rewards) = self
            .claim_and_compute_user_rewards(&payment, farm_address.clone(), &mut farm_state_mapper)
            .into_tuple();

        farm_state_mapper.update(|config| {
            config.farm_staked_value -= &payment.amount;
            config.farm_unstaked_value += &payment.amount;
        });
        let current_epoch = self.blockchain().get_block_epoch();
        let unstake_attributes = UnstakeFarmAttributes {
            farm_address,
            unstake_epoch: current_epoch,
            token_nonce: new_farm_token.token_nonce,
        };
        let unstake_token_payment = self
            .unstake_farm_token()
            .nft_create(payment.amount.clone(), &unstake_attributes);

        let mut user_payments = ManagedVec::from_single_item(unstake_token_payment);
        if user_rewards.amount > 0 {
            let wrapper_user_rewards = self.wrap_locked_token(user_rewards);
            user_payments.push(wrapper_user_rewards);
        }
        let caller = self.blockchain().get_caller();
        self.send().direct_multi(&caller, &user_payments);

        user_payments
    }

    #[payable("*")]
    #[endpoint(unbondFarm)]
    fn unbond_farm(&self) -> PaymentsVec<Self::Api> {
        let payment = self.call_value().single_esdt();
        require!(
            payment.token_identifier == self.unstake_farm_token().get_token_id(),
            ERROR_BAD_PAYMENT_TOKENS
        );
        let token_attributes: UnstakeFarmAttributes<Self::Api> =
            self.get_token_attributes(&payment.token_identifier, payment.token_nonce);
        let farm_address = token_attributes.farm_address;
        let mut farm_state_mapper = self.farm_state(&farm_address);
        require!(!farm_state_mapper.is_empty(), ERROR_FARM_DOES_NOT_EXIST);

        let current_epoch = self.blockchain().get_block_epoch();
        let unbond_period = self.get_minimum_farming_epochs(farm_address.clone());
        let unbond_epoch = token_attributes.unstake_epoch + unbond_period;
        require!(current_epoch >= unbond_epoch, ERROR_UNBOND_TOO_SOON);

        let farm_token_id = self.get_farm_token(farm_address.clone());
        let unstake_payment = EsdtTokenPayment::new(
            farm_token_id.clone(),
            token_attributes.token_nonce,
            payment.amount.clone(),
        );

        // Needed in order to claim all the boosted rewards, in case they were distributed
        let empty_payment = EsdtTokenPayment::new(farm_token_id, 0u64, BigUint::zero());
        self.claim_and_compute_user_rewards(
            &empty_payment,
            farm_address.clone(),
            &mut farm_state_mapper,
        );

        let exit_farm_result = self.call_exit_farm(farm_address, unstake_payment);
        let (mut farming_tokens, locked_rewards_payment) = exit_farm_result.into_tuple();

        farm_state_mapper.update(|config| {
            config.farm_unstaked_value -= &payment.amount;
        });

        self.send().esdt_local_burn(
            &payment.token_identifier,
            payment.token_nonce,
            &payment.amount,
        );
        self.apply_fee(&mut farming_tokens);
        let mut user_payments = ManagedVec::from_single_item(farming_tokens);
        if locked_rewards_payment.amount > 0 {
            user_payments.push(locked_rewards_payment);
        }
        let caller = self.blockchain().get_caller();
        self.send().direct_multi(&caller, &user_payments);

        user_payments
    }

    fn claim_and_compute_user_rewards(
        &self,
        payment: &EsdtTokenPayment<Self::Api>,
        farm_address: ManagedAddress,
        farm_state_mapper: &mut SingleValueMapper<FarmState<Self::Api>>,
    ) -> ClaimRewardsResultType<Self::Api> {
        let farm_state = farm_state_mapper.get();
        if farm_state.farm_staked_value == 0 {
            return (payment.clone(), payment.clone()).into();
        }

        let division_safety_constant = self.get_division_safety_constant(farm_address.clone());
        let farm_token_id = self.get_farm_token(farm_address.clone());
        let current_farm_position = EsdtTokenPayment::new(
            farm_token_id,
            farm_state.farm_token_nonce,
            farm_state.farm_staked_value.clone(),
        );
        let claim_rewards_result = self.call_farm_claim(farm_address, current_farm_position);
        let (new_farm_token, farm_rewards) = claim_rewards_result.into_tuple();

        self.update_farm_after_claim(
            &farm_state,
            farm_state_mapper,
            &new_farm_token,
            farm_rewards,
            &division_safety_constant,
        );

        // The contract uses MetaESDTs for user positions, and only these need to be burned
        // Empty simulated payments are used to claim rewards and have an up-to-date farm state, for proper user position computations
        if payment.amount > 0 {
            self.send().esdt_local_burn(
                &payment.token_identifier,
                payment.token_nonce,
                &payment.amount,
            );
        }
        let user_rewards = self.compute_user_rewards_payment(
            farm_state_mapper,
            payment,
            &division_safety_constant,
        );
        farm_state_mapper.update(|config| config.reward_reserve -= &user_rewards.amount);

        (new_farm_token, user_rewards).into()
    }
}
