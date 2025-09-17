use crate::action_type::DeployActionType;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait FeeModule: multiversx_sc_modules::pause::PauseModule {
    #[only_owner]
    #[endpoint(setActionFee)]
    fn set_action_fee(
        &self,
        type_amount_pairs: MultiValueEncoded<MultiValue2<DeployActionType, BigUint>>,
    ) {
        self.require_paused();

        for pair in type_amount_pairs {
            let (action_type, fee_amount) = pair.into_tuple();
            require!(action_type != DeployActionType::None, "Invalid action type");
            self.require_non_zero_fee(&fee_amount);

            self.custom_action_fee(action_type).set(fee_amount);
        }
    }

    #[only_owner]
    #[endpoint(setDefaultActionFee)]
    fn set_default_action_fee(&self, default_action_fee: BigUint) {
        self.require_paused();
        self.require_non_zero_fee(&default_action_fee);

        self.default_action_fee().set(default_action_fee);
    }

    #[only_owner]
    #[endpoint(claimFees)]
    fn claim_fees(&self) {
        let total_fees = self.total_fees().take();
        if total_fees == 0 {
            return;
        }

        let caller = self.blockchain().get_caller();
        let fees_token_id = self.fee_token().get();
        self.send()
            .direct_esdt(&caller, &fees_token_id, 0, &total_fees);
    }

    #[view(getActionFee)]
    fn get_action_fee(&self, action_type: DeployActionType) -> BigUint {
        let custom_fee = self.custom_action_fee(action_type).get();
        if custom_fee > 0 {
            return custom_fee;
        }

        self.default_action_fee().get()
    }

    fn take_fee(
        &self,
        caller: &ManagedAddress,
        mut payment: EsdtTokenPayment,
        action_type: DeployActionType,
    ) {
        let fee_token = self.fee_token().get();
        require!(
            payment.token_identifier == fee_token,
            "Invalid token for fees"
        );

        let fee_for_action = self.get_action_fee(action_type);
        require!(
            payment.amount >= fee_for_action,
            "Not enough tokens for fee"
        );

        payment.amount -= &fee_for_action;

        self.total_fees().update(|total| *total += fee_for_action);

        self.send().direct_non_zero_esdt_payment(caller, &payment);
    }

    fn require_non_zero_fee(&self, fee_amount: &BigUint) {
        require!(fee_amount > &0, "Cannot set fee to 0");
    }

    #[view(getFeeToken)]
    #[storage_mapper("feeToken")]
    fn fee_token(&self) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("customActionFee")]
    fn custom_action_fee(&self, action_type: DeployActionType) -> SingleValueMapper<BigUint>;

    #[storage_mapper("defaultActionFee")]
    fn default_action_fee(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("totalFees")]
    fn total_fees(&self) -> SingleValueMapper<BigUint>;
}
