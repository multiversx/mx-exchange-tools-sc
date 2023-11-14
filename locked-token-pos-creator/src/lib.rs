#![no_std]

multiversx_sc::imports!();

pub mod create_farm_pos;
pub mod create_pair_pos;
pub mod create_pos;
pub mod external_sc_interactions;

use auto_pos_creator::configs::{self, pairs_config::SwapOperationType};
use common_structs::Epoch;

#[multiversx_sc::contract]
pub trait LockedTokenPosCreatorContract:
    create_pos::CreatePosModule
    + create_pair_pos::CreatePairPosModule
    + create_farm_pos::CreateFarmPosModule
    + external_sc_interactions::egld_wrapper_actions::EgldWrapperActionsModule
    + external_sc_interactions::energy_factory_actions::EnergyFactoryActionsModule
    + external_sc_interactions::proxy_dex_actions::ProxyDexActionsModule
    + energy_query::EnergyQueryModule
    + utils::UtilsModule
    + configs::pairs_config::PairsConfigModule
    + auto_pos_creator::external_sc_interactions::pair_actions::PairActionsModule
    + auto_pos_creator::external_sc_interactions::router_actions::RouterActionsModule
{
    /// This contract needs the burn role for MEX token
    #[init]
    fn init(
        &self,
        energy_factory_adddress: ManagedAddress,
        egld_wrapper_address: ManagedAddress,
        mex_wegld_lp_pair_address: ManagedAddress,
        mex_wegld_lp_farm_address: ManagedAddress,
        proxy_dex_address: ManagedAddress,
        router_address: ManagedAddress,
    ) {
        self.require_sc_address(&egld_wrapper_address);
        self.require_sc_address(&mex_wegld_lp_pair_address);
        self.require_sc_address(&mex_wegld_lp_farm_address);
        self.require_sc_address(&proxy_dex_address);
        self.require_sc_address(&router_address);

        self.egld_wrapper_sc_address().set(egld_wrapper_address);
        self.pair_address().set(mex_wegld_lp_pair_address);
        self.farm_address().set(mex_wegld_lp_farm_address);
        self.proxy_dex_address().set(proxy_dex_address);
        self.router_address().set(router_address);

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
        swap_operations: MultiValueEncoded<SwapOperationType<Self::Api>>,
    ) -> EsdtTokenPayment {
        let caller = self.blockchain().get_caller();
        let payment = self.call_value().egld_or_single_esdt();
        let mex_payment = self.process_payment(payment, swap_operations);

        let output_payment = self.call_lock_virtual(mex_payment, lock_epochs, caller.clone());

        require!(output_payment.amount >= min_amount_out, "Slippage exceeded");

        self.send().direct_esdt(
            &caller,
            &output_payment.token_identifier,
            output_payment.token_nonce,
            &output_payment.amount,
        );

        output_payment
    }
}
