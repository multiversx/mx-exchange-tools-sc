#![no_std]

elrond_wasm::imports!();

pub mod address_to_id_mapper;
pub mod common_storage;
pub mod farm_external_storage_read;
pub mod farms_whitelist;
pub mod user_farm_actions;
pub mod user_farm_tokens;

#[elrond_wasm::contract]
pub trait AutoFarm:
    farms_whitelist::FarmsWhitelistModule
    + farm_external_storage_read::FarmExternalStorageReadModule
    + common_storage::CommonStorageModule
    + user_farm_tokens::UserFarmTokensModule
    + user_farm_actions::UserFarmActionsModule
    + utils::UtilsModule
{
    /// Arg: The address that can call the claim endpoints for users
    #[init]
    fn init(&self, proxy_claim_address: ManagedAddress) {
        self.proxy_claim_address().set(&proxy_claim_address);
    }
}
