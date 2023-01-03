use common_structs::PaymentsVec;
use metabonding::claim::{ClaimArgPair, ProxyTrait as _};

use crate::user_rewards::UniquePayments;

elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait MetabondingActionsModule:
    crate::common_storage::CommonStorageModule
    + crate::user_rewards::UserRewardsModule
    + crate::fees::FeesModule
    + utils::UtilsModule
    + energy_query::EnergyQueryModule
    + crate::locked_token_merging::LockedTokenMergingModule
    + lkmex_transfer::energy_transfer::EnergyTransferModule
    + legacy_token_decode_module::LegacyTokenDecodeModule
{
    #[endpoint(claimMetabondingRewards)]
    fn claim_metabonding_rewards(
        &self,
        user: ManagedAddress,
        claim_args: MultiValueEncoded<ClaimArgPair<Self::Api>>,
    ) {
        self.require_caller_proxy_claim_address();

        let rewards = self.call_metabonding_claim(user.clone(), claim_args);
        if rewards.is_empty() {
            return;
        }

        let merged_rewards = UniquePayments::new_from_payments(rewards);
        let user_id = self.user_ids().get_id(&user);
        self.add_user_rewards(user, user_id, UniquePayments::new(), merged_rewards);
    }

    fn call_metabonding_claim(
        &self,
        user: ManagedAddress,
        claim_args: MultiValueEncoded<ClaimArgPair<Self::Api>>,
    ) -> PaymentsVec<Self::Api> {
        let sc_address = self.metabonding_sc_address().get();
        self.metabonding_proxy(sc_address)
            .claim_rewards(user, claim_args)
            .execute_on_dest_context()
    }

    #[proxy]
    fn metabonding_proxy(&self, sc_address: ManagedAddress) -> metabonding::Proxy<Self::Api>;

    #[storage_mapper("metabondingScAddress")]
    fn metabonding_sc_address(&self) -> SingleValueMapper<ManagedAddress>;
}
