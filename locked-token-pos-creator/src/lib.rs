#![no_std]

multiversx_sc::imports!();

pub mod create_farm_pos;
pub mod create_pair_pos;
pub mod external_sc_interactions;

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
{
    #[init]
    fn init(
        &self,
        energy_factory_adddress: ManagedAddress,
        egld_wrapper_address: ManagedAddress,
        wegld_token_id: TokenIdentifier,
        mex_wegld_pair_address: ManagedAddress,
        mex_wegld_lp_farm_address: ManagedAddress,
        proxy_dex_address: ManagedAddress,
    ) {
        self.require_sc_address(&egld_wrapper_address);
        self.require_valid_token_id(&wegld_token_id);
        self.require_sc_address(&mex_wegld_pair_address);
        self.require_sc_address(&mex_wegld_lp_farm_address);
        self.require_sc_address(&proxy_dex_address);

        self.egld_wrapper_sc_address().set(egld_wrapper_address);
        self.wegld_token_id().set(wegld_token_id);
        self.mex_wegld_pair_address().set(mex_wegld_pair_address);
        self.farm_address().set(mex_wegld_lp_farm_address);
        self.proxy_dex_address().set(proxy_dex_address);

        self.set_energy_factory_address(energy_factory_adddress);
    }

    #[endpoint]
    fn upgrade(&self) {}
}
