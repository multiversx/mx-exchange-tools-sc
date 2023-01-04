#![no_std]

elrond_wasm::imports!();

pub mod address_to_id_mapper;
pub mod common_storage;
pub mod farm_actions;
pub mod farm_external_storage_read;
pub mod farms_whitelist;
pub mod fees;
pub mod fees_collector_actions;
pub mod locked_token_merging;
pub mod metabonding_actions;
pub mod user_farm_tokens;
pub mod user_rewards;

use common_storage::MAX_PERCENTAGE;

#[elrond_wasm::contract]
pub trait AutoFarm:
    farms_whitelist::FarmsWhitelistModule
    + farm_external_storage_read::FarmExternalStorageReadModule
    + common_storage::CommonStorageModule
    + user_farm_tokens::UserFarmTokensModule
    + farm_actions::FarmActionsModule
    + metabonding_actions::MetabondingActionsModule
    + fees_collector_actions::FeesCollectorActionsModule
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
        metabonding_sc_address: ManagedAddress,
        fees_collector_sc_address: ManagedAddress,
    ) {
        require!(
            fee_percentage > 0 && fee_percentage < MAX_PERCENTAGE,
            "Invalid fees percentage"
        );
        self.require_sc_address(&energy_factory_address);
        self.require_sc_address(&metabonding_sc_address);
        self.require_sc_address(&fees_collector_sc_address);

        self.proxy_claim_address().set_if_empty(proxy_claim_address);
        self.fee_percentage().set(fee_percentage);
        self.energy_factory_address()
            .set_if_empty(energy_factory_address);
        self.metabonding_sc_address()
            .set_if_empty(metabonding_sc_address);
        self.fees_collector_sc_address()
            .set_if_empty(fees_collector_sc_address);
    }

    #[only_owner]
    #[endpoint(changeProxyClaimAddress)]
    fn change_proxy_claim_address(&self, new_proxy_claim_address: ManagedAddress) {
        let old_claim_address = self.proxy_claim_address().replace(&new_proxy_claim_address);
        let unclaimed_tokens = self.accumulated_fees().get();
        if let Some(locked_tokens) = unclaimed_tokens.opt_locked_tokens {
            let tokens_vec = ManagedVec::from_single_item(locked_tokens);
            self.deduct_energy_from_sender(old_claim_address, &tokens_vec);
            self.add_energy_to_destination(new_proxy_claim_address, &tokens_vec);
        }
    }

    #[endpoint]
    fn register(&self) {
        let caller = self.blockchain().get_caller();
        let _ = self.user_ids().insert_new(&caller);
    }
}
