use auto_farm::common::unique_payments::UniquePayments;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub static NO_FUNDS_ERR_MSG: &[u8] = b"No funds deposited";

pub type Nonce = u64;

#[multiversx_sc::module]
pub trait UserFundsModule: utils::UtilsModule + multiversx_sc_modules::pause::PauseModule {
    #[payable("*")]
    #[endpoint]
    fn deposit(&self) {
        self.require_not_paused();

        let caller = self.blockchain().get_caller();
        let user_id = self.user_ids().get_id_or_insert(&caller);

        let egld_value = self.call_value().egld_value().clone_value();
        require!(egld_value == 0, "EGLD not accepted");

        let esdt_transfers = self.get_non_empty_payments();
        let user_funds_mapper = self.user_funds(user_id);
        if !user_funds_mapper.is_empty() {
            user_funds_mapper.update(|user_funds| {
                for payment in &esdt_transfers {
                    user_funds.add_payment(payment);
                }
            });
        } else {
            user_funds_mapper.set(UniquePayments::new_from_payments(esdt_transfers));
        }
    }

    #[endpoint]
    fn withdraw(&self, esdt: MultiValueEncoded<MultiValue3<TokenIdentifier, Nonce, BigUint>>) {
        self.require_not_paused();

        let caller = self.blockchain().get_caller();
        let user_id = self.user_ids().get_id_non_zero(&caller);
        let user_funds_mapper = self.user_funds(user_id);
        require!(!user_funds_mapper.is_empty(), NO_FUNDS_ERR_MSG);

        let mut esdt_withdrawn = UniquePayments::new();
        let any_funds_left = user_funds_mapper.update(|user_funds| {
            for multi_value_esdt in esdt {
                let (token_id, nonce, amount) = multi_value_esdt.into_tuple();
                let esdt_transfer = EsdtTokenPayment::new(token_id, nonce, amount);

                let deduct_result = user_funds.deduct_payment(&esdt_transfer);
                require!(deduct_result.is_ok(), "Withdrawing too much ESDT");

                esdt_withdrawn.add_payment(esdt_transfer);
            }

            user_funds.is_empty()
        });
        if !any_funds_left {
            user_funds_mapper.clear();
        }

        let esdt_as_vec = esdt_withdrawn.into_payments();
        if !esdt_as_vec.is_empty() {
            self.send().direct_multi(&caller, &esdt_as_vec);
        }
    }

    #[endpoint(withdrawAll)]
    fn withdraw_all(&self) {
        self.require_not_paused();

        let caller = self.blockchain().get_caller();
        let user_id = self.user_ids().get_id_non_zero(&caller);
        let user_funds_mapper = self.user_funds(user_id);
        require!(!user_funds_mapper.is_empty(), NO_FUNDS_ERR_MSG);

        let user_funds = user_funds_mapper.take();
        let esdt_as_vec = user_funds.into_payments();
        if !esdt_as_vec.is_empty() {
            self.send().direct_multi(&caller, &esdt_as_vec);
        }
    }

    #[view(getUserFunds)]
    fn get_user_funds(&self, user: ManagedAddress) -> OptionalValue<UniquePayments<Self::Api>> {
        let user_id = self.user_ids().get_id_non_zero(&user);
        let user_funds_mapper = self.user_funds(user_id);
        if !user_funds_mapper.is_empty() {
            OptionalValue::Some(user_funds_mapper.get())
        } else {
            OptionalValue::None
        }
    }

    #[storage_mapper("userId")]
    fn user_ids(&self) -> AddressToIdMapper<Self::Api>;

    #[storage_mapper("userFunds")]
    fn user_funds(&self, user_id: AddressId) -> SingleValueMapper<UniquePayments<Self::Api>>;
}
