use common_structs::{PaymentsVec, Week};
use metabonding::{
    claim::{ClaimArgPair, ProxyTrait as _},
    validation::Signature,
};

use crate::common::rewards_wrapper::RewardsWrapper;

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem)]
pub struct SingleMetabondingClaimArg<M: ManagedTypeApi> {
    pub week: Week,
    pub user_delegation_amount: BigUint<M>,
    pub user_lkmex_amount: BigUint<M>,
    pub signature: Signature<M>,
}

#[elrond_wasm::module]
pub trait MetabondingActionsModule:
    crate::common::common_storage::CommonStorageModule
    + crate::user_tokens::user_rewards::UserRewardsModule
    + crate::user_tokens::withdraw_tokens::WithdrawTokensModule
    + crate::fees::FeesModule
    + crate::events::EventsModule
    + utils::UtilsModule
    + energy_query::EnergyQueryModule
    + crate::external_sc_interactions::locked_token_merging::LockedTokenMergingModule
    + lkmex_transfer::energy_transfer::EnergyTransferModule
    + legacy_token_decode_module::LegacyTokenDecodeModule
{
    fn claim_metabonding_rewards(
        &self,
        user: &ManagedAddress,
        claim_args: ManagedVec<SingleMetabondingClaimArg<Self::Api>>,
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
        claim_args: ManagedVec<SingleMetabondingClaimArg<Self::Api>>,
    ) -> PaymentsVec<Self::Api> {
        let mut formatted_claim_args = MultiValueEncoded::new();
        for claim_arg in &claim_args {
            let formatted_arg: ClaimArgPair<Self::Api> = (
                claim_arg.week,
                claim_arg.user_delegation_amount,
                claim_arg.user_lkmex_amount,
                claim_arg.signature,
            )
                .into();
            formatted_claim_args.push(formatted_arg);
        }

        let sc_address = self.metabonding_sc_address().get();
        self.metabonding_proxy(sc_address)
            .claim_rewards(user, formatted_claim_args)
            .execute_on_dest_context()
    }

    #[proxy]
    fn metabonding_proxy(&self, sc_address: ManagedAddress) -> metabonding::Proxy<Self::Api>;

    #[storage_mapper("metabondingScAddress")]
    fn metabonding_sc_address(&self) -> SingleValueMapper<ManagedAddress>;
}
