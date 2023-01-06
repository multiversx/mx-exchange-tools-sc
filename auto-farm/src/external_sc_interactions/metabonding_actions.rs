use common_structs::PaymentsVec;
use metabonding::claim::{ClaimArgPair, ProxyTrait as _};

use crate::common::rewards_wrapper::RewardsWrapper;

elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait MetabondingActionsModule:
    crate::common::common_storage::CommonStorageModule
    + crate::user_tokens::user_rewards::UserRewardsModule
    + crate::fees::FeesModule
    + utils::UtilsModule
    + energy_query::EnergyQueryModule
    + crate::external_sc_interactions::locked_token_merging::LockedTokenMergingModule
    + lkmex_transfer::energy_transfer::EnergyTransferModule
    + legacy_token_decode_module::LegacyTokenDecodeModule
{
    fn claim_metabonding_rewards(
        &self,
        user: &ManagedAddress,
        claim_args: MultiValueEncoded<ClaimArgPair<Self::Api>>,
        rew_wrapper: &mut RewardsWrapper<Self::Api>,
    ) {
        if claim_args.is_empty() {
            return;
        }

        let rewards = self.call_metabonding_claim(user.clone(), claim_args);
        for rew in &rewards {
            rew_wrapper.other_tokens.add_payment(rew);
        }
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
