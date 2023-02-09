multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use common_structs::{Epoch, Nonce};

use crate::common::errors::{
    ERROR_DIVISION_CONSTANT_VALUE, ERROR_FARM_ALREADY_DEFINED, ERROR_FARM_DOES_NOT_EXIST,
    ERROR_FARM_HAS_FUNDS,
};

#[derive(
    TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Copy, Clone, Debug,
)]
pub enum State {
    Inactive,
    Active,
    PartialActive,
}

#[derive(TypeAbi, TopEncode, TopDecode, Debug)]
pub struct FarmState<M: ManagedTypeApi> {
    pub farm_staked_value: BigUint<M>,
    pub farm_token_nonce: Nonce,
    pub reward_token_nonce: Nonce,
    pub farm_unstaked_value: BigUint<M>,
    pub reward_reserve: BigUint<M>,
    pub farm_rps: BigUint<M>,
}

#[derive(TypeAbi, TopEncode, TopDecode, Debug, PartialEq)]
pub struct WrappedFarmTokenAttributes<M: ManagedTypeApi> {
    pub token_rps: BigUint<M>,
}

#[derive(TypeAbi, TopEncode, TopDecode, Debug, PartialEq)]
pub struct UnstakeTokenAttributes {
    pub unstake_epoch: Epoch,
    pub token_nonce: Nonce,
}

pub type PaymentsVec<M> = ManagedVec<M, EsdtTokenPayment<M>>;

pub const MAX_PERCENT: u64 = 10_000;

#[multiversx_sc::module]
pub trait FarmConfigModule: utils::UtilsModule {
    /// Allows the setup of farms in the contract. For each farm, the following values are required:
    /// farm address: the address of the farm
    /// wrapped_farm_token_id: token id previously issued, that will reflect the farm position of the users (roles must be already set)
    /// unstake_farm_token_id: token id previously issued, that will reflect the unstake position of the users (roles must be already set)
    #[only_owner]
    #[endpoint(addFarms)]
    fn add_farms(
        &self,
        farms: MultiValueEncoded<MultiValue3<ManagedAddress, TokenIdentifier, TokenIdentifier>>,
    ) {
        for farm in farms {
            let (farm_addr, wrapped_farm_token_id, unstake_farm_token_id) = farm.into_tuple();
            let farm_state_mapper = self.farm_state(&farm_addr);
            require!(farm_state_mapper.is_empty(), ERROR_FARM_ALREADY_DEFINED);
            self.require_sc_address(&farm_addr);
            self.require_valid_token_id(&wrapped_farm_token_id);
            self.require_valid_token_id(&unstake_farm_token_id);
            self.wrapped_farm_token_id().set(wrapped_farm_token_id);
            self.unstake_farm_token_id().set(unstake_farm_token_id);

            let farm_state = FarmState {
                farm_staked_value: BigUint::zero(),
                farm_token_nonce: 0u64,
                reward_token_nonce: 0u64,
                farm_unstaked_value: BigUint::zero(),
                reward_reserve: BigUint::zero(),
                farm_rps: BigUint::zero(),
            };
            farm_state_mapper.set(farm_state);
        }
    }

    #[only_owner]
    #[endpoint(removeFarms)]
    fn remove_farms(&self, farms: MultiValueEncoded<ManagedAddress>) {
        for farm in farms {
            let farm_state_mapper = self.farm_state(&farm);
            require!(!farm_state_mapper.is_empty(), ERROR_FARM_DOES_NOT_EXIST);
            let farm_state = farm_state_mapper.get();
            require!(farm_state.farm_staked_value == 0, ERROR_FARM_HAS_FUNDS);
            farm_state_mapper.clear();
        }
    }

    fn mint_tokens<T: TopEncode>(
        &self,
        token_id: TokenIdentifier,
        amount: BigUint,
        attributes: &T,
    ) -> EsdtTokenPayment<Self::Api> {
        let new_nonce = self
            .send()
            .esdt_nft_create_compact(&token_id, &amount, attributes);

        EsdtTokenPayment::new(token_id, new_nonce, amount)
    }

    fn get_token_attributes<T: TopDecode>(
        &self,
        token_id: &TokenIdentifier,
        token_nonce: u64,
    ) -> T {
        let token_info = self.blockchain().get_esdt_token_data(
            &self.blockchain().get_sc_address(),
            token_id,
            token_nonce,
        );

        token_info.decode_attributes()
    }

    #[view(getFarmState)]
    fn get_farm_state(&self, farm_address: &ManagedAddress) -> FarmState<Self::Api> {
        let farm_state_mapper = self.farm_state(farm_address);
        require!(!farm_state_mapper.is_empty(), "Farm does not exist");
        farm_state_mapper.get()
    }

    fn get_farming_token(&self, farm_address: &ManagedAddress) -> TokenIdentifier {
        let farming_token_id = self.farming_token_id().get_from_address(farm_address);
        self.require_valid_token_id(&farming_token_id);
        farming_token_id
    }

    fn get_farm_token(&self, farm_address: &ManagedAddress) -> TokenIdentifier {
        let farm_token_id = self.farm_token_id().get_from_address(farm_address);
        self.require_valid_token_id(&farm_token_id);
        farm_token_id
    }

    fn get_division_safety_constant(&self, farm_address: &ManagedAddress) -> BigUint {
        let division_safety_constant = self
            .division_safety_constant()
            .get_from_address(farm_address);
        require!(division_safety_constant > 0, ERROR_DIVISION_CONSTANT_VALUE);
        division_safety_constant
    }

    #[view(getFarmTokenId)]
    #[storage_mapper("farm_token_id")]
    fn farm_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getFarmingTokenId)]
    #[storage_mapper("farming_token_id")]
    fn farming_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getDivisionSafetyConstant)]
    #[storage_mapper("division_safety_constant")]
    fn division_safety_constant(&self) -> SingleValueMapper<BigUint>;

    #[view(getWrappedFarmTokenId)]
    #[storage_mapper("wrappedFarmTokenId")]
    fn wrapped_farm_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getUnstakeFarmTokenId)]
    #[storage_mapper("unstakeFarmTokenId")]
    fn unstake_farm_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getUnbondPeriod)]
    #[storage_mapper("unbondPeriod")]
    fn unbond_period(&self) -> SingleValueMapper<Epoch>;

    #[view(getPenaltyPercent)]
    #[storage_mapper("penaltyPercent")]
    fn penalty_percent(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("exitFees")]
    fn exit_fees(&self) -> SingleValueMapper<EsdtTokenPayment>;

    #[storage_mapper("farmState")]
    fn farm_state(&self, farm_address: &ManagedAddress) -> SingleValueMapper<FarmState<Self::Api>>;
}
