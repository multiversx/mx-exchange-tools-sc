#![no_std]

multiversx_sc::imports!();

pub mod common;
pub mod events;
pub mod external_sc_interactions;
pub mod external_storage_read;
pub mod fees;
pub mod registration;
pub mod user_tokens;
pub mod whitelists;

use common::common_storage::MAX_PERCENTAGE;

#[multiversx_sc::contract]
pub trait AutoFarm:
    read_external_storage::ReadExternalStorageModule
    + whitelists::farms_whitelist::FarmsWhitelistModule
    + external_storage_read::farm_storage_read::FarmStorageReadModule
    + common::common_storage::CommonStorageModule
    + registration::RegistrationModule
    + user_tokens::user_farm_tokens::UserFarmTokensModule
    + external_sc_interactions::farm_actions::FarmActionsModule
    + external_sc_interactions::metabonding_actions::MetabondingActionsModule
    + external_sc_interactions::fees_collector_actions::FeesCollectorActionsModule
    + external_sc_interactions::locked_token_merging::LockedTokenMergingModule
    + whitelists::metastaking_whitelist::MetastakingWhitelistModule
    + user_tokens::user_metastaking_tokens::UserMetastakingTokensModule
    + external_storage_read::metastaking_storage_read::MetastakingStorageReadModule
    + external_sc_interactions::metastaking_actions::MetastakingActionsModule
    + external_sc_interactions::multi_contract_interactions::MultiContractInteractionsModule
    + user_tokens::user_rewards::UserRewardsModule
    + user_tokens::withdraw_tokens::WithdrawTokensModule
    + fees::FeesModule
    + events::EventsModule
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

    #[upgrade]
    fn upgrade(&self) {}

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
}
