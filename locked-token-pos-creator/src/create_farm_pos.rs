multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::create_pair_pos::AddLiquidityArguments;
use auto_pos_creator::configs;
use common_structs::Epoch;

#[derive(TypeAbi, TopEncode, TopDecode)]
pub struct CreateFarmPosResult<M: ManagedTypeApi> {
    pub wrapped_farm_token: EsdtTokenPayment<M>,
    pub rewards: EsdtTokenPayment<M>,
    pub locked_token_leftover: EsdtTokenPayment<M>,
    pub wegld_leftover: EsdtTokenPayment<M>,
}

#[multiversx_sc::module]
pub trait CreateFarmPosModule:
    crate::external_sc_interactions::egld_wrapper_actions::EgldWrapperActionsModule
    + crate::external_sc_interactions::energy_factory_actions::EnergyFactoryActionsModule
    + crate::external_sc_interactions::pair_actions::PairActionsModule
    + crate::external_sc_interactions::proxy_dex_actions::ProxyDexActionsModule
    + crate::create_pair_pos::CreatePairPosModule
    + energy_query::EnergyQueryModule
    + utils::UtilsModule
    + configs::pairs_config::PairsConfigModule
{
    #[payable("*")]
    #[endpoint(createFarmPosFromSingleToken)]
    fn create_farm_pos_from_single_token(
        &self,
        swap_min_amount_out: BigUint,
        lock_epochs: Epoch,
        add_liq_first_token_min_amount: BigUint,
        add_liq_second_token_min_amount: BigUint,
    ) -> CreateFarmPosResult<Self::Api> {
        let payment = self.call_value().egld_or_single_esdt();
        let wegld_payment = self.get_wegld_payment(payment);
        let args = AddLiquidityArguments {
            payment: wegld_payment,
            swap_min_amount_out,
            lock_epochs,
            add_liq_first_token_min_amount,
            add_liq_second_token_min_amount,
        };

        let add_liq_result = self.create_pair_pos_from_single_token(args);

        let mut output_payments = ManagedVec::new();
        if add_liq_result.locked_token_leftover.amount > 0 {
            output_payments.push(add_liq_result.locked_token_leftover.clone());
        }
        if add_liq_result.wegld_leftover.amount > 0 {
            output_payments.push(add_liq_result.wegld_leftover.clone());
        }

        let caller = self.blockchain().get_caller();
        let farm_address = self.farm_address().get();
        let enter_result = self.call_enter_farm_proxy(
            caller.clone(),
            add_liq_result.wrapped_lp_token,
            farm_address,
        );

        output_payments.push(enter_result.wrapped_farm_token.clone());

        if enter_result.rewards.amount > 0 {
            output_payments.push(enter_result.rewards.clone());
        }

        self.send().direct_multi(&caller, &output_payments);

        CreateFarmPosResult {
            wegld_leftover: add_liq_result.wegld_leftover,
            locked_token_leftover: add_liq_result.locked_token_leftover,
            wrapped_farm_token: enter_result.wrapped_farm_token,
            rewards: enter_result.rewards,
        }
    }

    /// Create pos from two payments, by adding liquidity with the provided tokens
    /// It only accepts locked token and wrapped egld payments
    #[payable("*")]
    #[endpoint(createFarmPosFromTwoTokens)]
    fn create_farm_pos_from_two_tokens(
        &self,
        add_liq_first_token_min_amount: BigUint,
        add_liq_second_token_min_amount: BigUint,
    ) -> CreateFarmPosResult<Self::Api> {
        let caller = self.blockchain().get_caller();
        let [first_payment, second_payment] = self.call_value().multi_esdt();

        let locked_token_id = self.get_locked_token_id();
        let wegld_token_id = self.wegld_token_id().get();

        require!(
            first_payment.token_identifier == locked_token_id
                || first_payment.token_identifier == wegld_token_id,
            "Invalid payment tokens"
        );
        require!(
            second_payment.token_identifier == locked_token_id
                || second_payment.token_identifier == wegld_token_id,
            "Invalid payment tokens"
        );
        require!(
            first_payment.token_identifier != second_payment.token_identifier,
            "Invalid payment tokens"
        );

        let wrapped_dest_pair_address = self.get_pair_address_for_tokens(
            &first_payment.token_identifier,
            &second_payment.token_identifier,
        );

        let mut proxy_payments = ManagedVec::new();
        proxy_payments.push(first_payment);
        proxy_payments.push(second_payment);

        let add_liq_result = self.call_add_liquidity_proxy(
            proxy_payments,
            wrapped_dest_pair_address.unwrap_address(),
            add_liq_first_token_min_amount,
            add_liq_second_token_min_amount,
        );

        let mut output_payments = ManagedVec::new();
        if add_liq_result.locked_token_leftover.amount > 0 {
            output_payments.push(add_liq_result.locked_token_leftover.clone());
        }
        if add_liq_result.wegld_leftover.amount > 0 {
            output_payments.push(add_liq_result.wegld_leftover.clone());
        }

        let farm_address = self.farm_address().get();
        let enter_result = self.call_enter_farm_proxy(
            caller.clone(),
            add_liq_result.wrapped_lp_token,
            farm_address,
        );

        output_payments.push(enter_result.wrapped_farm_token.clone());

        if enter_result.rewards.amount > 0 {
            output_payments.push(enter_result.rewards.clone());
        }

        self.send().direct_multi(&caller, &output_payments);

        CreateFarmPosResult {
            wegld_leftover: add_liq_result.wegld_leftover,
            locked_token_leftover: add_liq_result.locked_token_leftover,
            wrapped_farm_token: enter_result.wrapped_farm_token,
            rewards: enter_result.rewards,
        }
    }

    #[storage_mapper("wegldMexLpFarmAddress")]
    fn farm_address(&self) -> SingleValueMapper<ManagedAddress>;
}
