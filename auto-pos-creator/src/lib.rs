#![no_std]

multiversx_sc::imports!();

pub mod common;
pub mod configs;
pub mod external_sc_interactions;
pub mod multi_contract_interactions;

#[multiversx_sc::contract]
pub trait AutoPosCreator:
    auto_farm::whitelists::farms_whitelist::FarmsWhitelistModule
    + auto_farm::whitelists::metastaking_whitelist::MetastakingWhitelistModule
    + auto_farm::external_storage_read::farm_storage_read::FarmStorageReadModule
    + auto_farm::external_storage_read::metastaking_storage_read::MetastakingStorageReadModule
    + utils::UtilsModule
    + configs::auto_farm_config::AutoFarmConfigModule
    + configs::pairs_config::PairsConfigModule
    + external_sc_interactions::pair_actions::PairActionsModule
    + external_sc_interactions::farm_actions::FarmActionsModule
    + external_sc_interactions::metastaking_actions::MetastakingActionsModule
    + multi_contract_interactions::create_pos::CreatePosModule
    + multi_contract_interactions::exit_pos::ExitPosModule
{
    /// Auto-farm SC is only used to read the farms and metastaking addresses from it.
    /// This way, we don't need to duplicate the setup in this SC as well
    #[init]
    fn init(&self, auto_farm_sc_address: ManagedAddress) {
        self.require_sc_address(&auto_farm_sc_address);

        self.auto_farm_sc_address()
            .set_if_empty(&auto_farm_sc_address);
    }
}
