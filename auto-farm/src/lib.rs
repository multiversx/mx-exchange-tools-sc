#![no_std]

elrond_wasm::imports!();

pub mod address_to_id_mapper;
pub mod common_storage;
pub mod farm_external_storage_read;
pub mod farms_whitelist;
pub mod fees;
pub mod locked_token_merging;
pub mod user_farm_actions;
pub mod user_farm_tokens;
pub mod user_rewards;

use common_storage::MAX_PERCENTAGE;

#[elrond_wasm::contract]
pub trait AutoFarm:
    farms_whitelist::FarmsWhitelistModule
    + farm_external_storage_read::FarmExternalStorageReadModule
    + common_storage::CommonStorageModule
    + user_farm_tokens::UserFarmTokensModule
    + user_farm_actions::UserFarmActionsModule
    + user_rewards::UserRewardsModule
    + fees::FeesModule
    + locked_token_merging::LockedTokenMergingModule
    + lkmex_transfer::energy_transfer::EnergyTransferModule
    + legacy_token_decode_module::LegacyTokenDecodeModule
    + energy_query::EnergyQueryModule
    + utils::UtilsModule
{
    /// proxy_claim_address: The address that can call the claim endpoints for users
    /// fee_percentage: The percentage of rewards that are taken as fees for every action.
    ///     Must be a value between 0 and 10_000, where 10_000 is 100%.
    /// energy_factory_address: SC address handling user energy
    #[init]
    fn init(
        &self,
        proxy_claim_address: ManagedAddress,
        fee_percentage: u64,
        energy_factory_address: ManagedAddress,
    ) {
        require!(
            fee_percentage > 0 && fee_percentage < MAX_PERCENTAGE,
            "Invalid fees percentage"
        );
        self.require_sc_address(&energy_factory_address);

        self.proxy_claim_address().set_if_empty(proxy_claim_address);
        self.fee_percentage().set(fee_percentage);
        self.energy_factory_address()
            .set_if_empty(energy_factory_address);
    }
}
