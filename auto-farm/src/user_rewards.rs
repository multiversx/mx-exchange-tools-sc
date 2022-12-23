use common_structs::PaymentsVec;
use mergeable::Mergeable;

use crate::address_to_id_mapper::AddressId;

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[derive(TypeAbi, TopEncode, TopDecode)]
pub struct RewardsWrapper<M: ManagedTypeApi> {
    pub opt_locked_tokens: Option<EsdtTokenPayment<M>>,
    pub other_tokens: UniquePayments<M>,
}

impl<M: ManagedTypeApi> Default for RewardsWrapper<M> {
    #[inline]
    fn default() -> Self {
        Self {
            opt_locked_tokens: None,
            other_tokens: UniquePayments::default(),
        }
    }
}

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, Clone)]
pub struct UniquePayments<M: ManagedTypeApi> {
    payments: PaymentsVec<M>,
}

impl<M: ManagedTypeApi> Default for UniquePayments<M> {
    #[inline]
    fn default() -> Self {
        Self {
            payments: PaymentsVec::new(),
        }
    }
}

impl<M: ManagedTypeApi> UniquePayments<M> {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn new_from_payments(payments: PaymentsVec<M>) -> Self {
        UniquePayments { payments }
    }

    pub fn add_payment(&mut self, new_payment: EsdtTokenPayment<M>) {
        if new_payment.amount == 0 {
            return;
        }

        let len = self.payments.len();
        for i in 0..len {
            let mut current_payment = self.payments.get(i);
            if current_payment.can_merge_with(&new_payment) {
                current_payment.amount += new_payment.amount;
                let _ = self.payments.set(i, &current_payment);

                return;
            }
        }

        self.payments.push(new_payment);
    }

    #[inline]
    pub fn into_payments(self) -> PaymentsVec<M> {
        self.payments
    }
}

impl<M: ManagedTypeApi> Mergeable<M> for UniquePayments<M> {
    #[inline]
    fn can_merge_with(&self, _other: &Self) -> bool {
        true
    }

    fn merge_with(&mut self, mut other: Self) {
        self.error_if_not_mergeable(&other);

        if self.payments.is_empty() {
            self.payments = other.payments;
            return;
        }
        if other.payments.is_empty() {
            return;
        }

        let first_len = self.payments.len();
        let mut second_len = other.payments.len();
        for i in 0..first_len {
            let mut current_payment = self.payments.get(i);
            for j in 0..second_len {
                let other_payment = other.payments.get(j);
                if !current_payment.can_merge_with(&other_payment) {
                    continue;
                }

                current_payment.amount += other_payment.amount;
                let _ = self.payments.set(i, &current_payment);

                other.payments.remove(j);
                second_len -= 1;

                break;
            }
        }

        self.payments.append_vec(other.payments);
    }
}

#[elrond_wasm::module]
pub trait UserRewardsModule:
    crate::common_storage::CommonStorageModule
    + crate::fees::FeesModule
    + crate::locked_token_merging::LockedTokenMergingModule
    + lkmex_transfer::energy_transfer::EnergyTransferModule
    + legacy_token_decode_module::LegacyTokenDecodeModule
    + energy_query::EnergyQueryModule
    + utils::UtilsModule
{
    #[endpoint(userClaimRewards)]
    fn user_claim_rewards(&self) -> PaymentsVec<Self::Api> {
        let caller = self.blockchain().get_caller();
        let user_id = self.user_ids().get_id_or_insert(&caller);
        let rewards_mapper = self.user_rewards(user_id);
        self.claim_common(caller, rewards_mapper)
    }

    fn add_user_rewards(
        &self,
        user: ManagedAddress,
        locked_tokens: UniquePayments<Self::Api>,
        other_tokens: UniquePayments<Self::Api>,
    ) {
        let opt_merged_locked_tokens =
            self.merge_locked_tokens(user.clone(), locked_tokens.into_payments());
        let mut rew_wrapper = RewardsWrapper {
            opt_locked_tokens: opt_merged_locked_tokens,
            other_tokens,
        };
        self.take_fees(user.clone(), &mut rew_wrapper);

        let user_id = self.user_ids().get_id_or_insert(&user);
        let rewards_mapper = self.user_rewards(user_id);
        if rewards_mapper.is_empty() {
            rewards_mapper.set(rew_wrapper);
            return;
        }

        rewards_mapper.update(|existing_wrapper| {
            if let Some(new_locked_tokens) = rew_wrapper.opt_locked_tokens {
                self.merge_wrapped_locked_tokens(user, existing_wrapper, new_locked_tokens);
            }

            existing_wrapper
                .other_tokens
                .merge_with(rew_wrapper.other_tokens);
        });
    }

    #[view(getUserRewards)]
    #[storage_mapper("userRewards")]
    fn user_rewards(&self, user_id: AddressId) -> SingleValueMapper<RewardsWrapper<Self::Api>>;
}
