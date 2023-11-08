#![no_std]

multiversx_sc::imports!();

pub mod create_farm_pos;
pub mod create_pair_pos;
pub mod external_sc_interactions;

use auto_pos_creator::configs;
use common_structs::Epoch;

#[multiversx_sc::contract]
pub trait LockedTokenPosCreatorContract:
    create_pair_pos::CreatePairPosModule
    + create_farm_pos::CreateFarmPosModule
    + external_sc_interactions::egld_wrapper_actions::EgldWrapperActionsModule
    + external_sc_interactions::energy_factory_actions::EnergyFactoryActionsModule
    + external_sc_interactions::pair_actions::PairActionsModule
    + external_sc_interactions::proxy_dex_actions::ProxyDexActionsModule
    + energy_query::EnergyQueryModule
    + utils::UtilsModule
    + configs::pairs_config::PairsConfigModule
{
    /// This contract needs the burn role for MEX token
    #[init]
    fn init(
        &self,
        energy_factory_adddress: ManagedAddress,
        egld_wrapper_address: ManagedAddress,
        wegld_token_id: TokenIdentifier,
        mex_wegld_lp_farm_address: ManagedAddress,
        proxy_dex_address: ManagedAddress,
    ) {
        self.require_sc_address(&egld_wrapper_address);
        self.require_valid_token_id(&wegld_token_id);
        self.require_sc_address(&mex_wegld_lp_farm_address);
        self.require_sc_address(&proxy_dex_address);

        self.egld_wrapper_sc_address().set(egld_wrapper_address);
        self.wegld_token_id().set(wegld_token_id);
        self.farm_address().set(mex_wegld_lp_farm_address);
        self.proxy_dex_address().set(proxy_dex_address);

        self.set_energy_factory_address(energy_factory_adddress);
    }

    #[endpoint]
    fn upgrade(&self) {}

    #[payable("*")]
    #[endpoint(createEnergyPosition)]
    fn create_energy_position(
        &self,
        lock_epochs: Epoch,
        min_amount_out: BigUint,
    ) -> EsdtTokenPayment {
        let caller = self.blockchain().get_caller();
        let payment = self.call_value().egld_or_single_esdt();
        let esdt_payment = self.get_esdt_payment(payment);
        let mex_token_id = self.get_base_token_id();
        let wegld_token_id = self.wegld_token_id().get();

        let output_payment = if esdt_payment.token_identifier == mex_token_id {
            self.call_lock_virtual(esdt_payment, lock_epochs, caller.clone())
        } else {
            let mex_pair_address = self
                .get_pair_address_for_tokens(&wegld_token_id, &mex_token_id)
                .unwrap_address();

            let wegld_payment = if esdt_payment.token_identifier == wegld_token_id {
                esdt_payment
            } else {
                let token_pair_address = self
                    .get_pair_address_for_tokens(&wegld_token_id, &esdt_payment.token_identifier)
                    .unwrap_address();

                self.call_pair_swap(token_pair_address, esdt_payment, wegld_token_id)
            };

            let mex_payment = self.call_pair_swap(mex_pair_address, wegld_payment, mex_token_id);
            require!(mex_payment.amount >= min_amount_out, "Slippage exceeded");

            self.call_lock_virtual(mex_payment, lock_epochs, caller.clone())
        };

        self.send().direct_esdt(
            &caller,
            &output_payment.token_identifier,
            output_payment.token_nonce,
            &output_payment.amount,
        );

        output_payment
    }
}
