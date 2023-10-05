use common_structs::Epoch;

use crate::external_sc_interactions::proxy_dex_actions::AddLiquidityProxyResult;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait CreateFarmPosModule:
    crate::external_sc_interactions::egld_wrapper_actions::EgldWrapperActionsModule
    + crate::external_sc_interactions::energy_factory_actions::EnergyFactoryActionsModule
    + crate::external_sc_interactions::pair_actions::PairActionsModule
    + crate::external_sc_interactions::proxy_dex_actions::ProxyDexActionsModule
    + crate::create_pair_pos::CreatePairPosModule
    + energy_query::EnergyQueryModule
    + utils::UtilsModule
{
}
