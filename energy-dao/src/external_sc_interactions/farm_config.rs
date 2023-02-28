multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use common_structs::{Epoch, Nonce};

use crate::common::errors::{
    ERROR_DIVISION_CONSTANT_VALUE, ERROR_FARM_ALREADY_DEFINED, ERROR_FARM_DOES_NOT_EXIST,
    ERROR_FARM_HAS_FUNDS,
};

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
    pub farm_address: ManagedAddress<M>,
    pub token_rps: BigUint<M>,
}

#[derive(TypeAbi, TopEncode, TopDecode, Debug, PartialEq)]
pub struct UnstakeTokenAttributes<M: ManagedTypeApi> {
    pub farm_address: ManagedAddress<M>,
    pub unstake_epoch: Epoch,
    pub token_nonce: Nonce,
}

pub type PaymentsVec<M> = ManagedVec<M, EsdtTokenPayment<M>>;

pub const MAX_PERCENT: u64 = 10_000;

#[multiversx_sc::module]
pub trait FarmConfigModule: utils::UtilsModule {
    #[only_owner]
    #[payable("EGLD")]
    #[endpoint(registerWrappedFarmToken)]
    fn register_wrapped_farm_token(
        &self,
        token_display_name: ManagedBuffer,
        token_ticker: ManagedBuffer,
        num_decimals: usize,
    ) {
        let payment_amount = self.call_value().egld_value();
        self.wrapped_farm_token().issue_and_set_all_roles(
            EsdtTokenType::Meta,
            payment_amount,
            token_display_name,
            token_ticker,
            num_decimals,
            None,
        );
    }

    #[only_owner]
    #[payable("EGLD")]
    #[endpoint(registerUnstakeFarmToken)]
    fn register_unstake_farm_token(
        &self,
        token_display_name: ManagedBuffer,
        token_ticker: ManagedBuffer,
        num_decimals: usize,
    ) {
        let payment_amount = self.call_value().egld_value();
        self.unstake_farm_token().issue_and_set_all_roles(
            EsdtTokenType::Meta,
            payment_amount,
            token_display_name,
            token_ticker,
            num_decimals,
            None,
        );
    }

    #[only_owner]
    #[endpoint(addFarms)]
    fn add_farms(&self, farms: MultiValueEncoded<ManagedAddress>) {
        for farm_addr in farms {
            let farm_state_mapper = self.farm_state(&farm_addr);
            require!(farm_state_mapper.is_empty(), ERROR_FARM_ALREADY_DEFINED);
            self.require_sc_address(&farm_addr);

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

    #[view(getFarmingTokenId)]
    fn get_farming_token(&self, farm_address: &ManagedAddress) -> TokenIdentifier {
        let farming_token_id = self.farming_token_id().get_from_address(farm_address);
        self.require_valid_token_id(&farming_token_id);
        farming_token_id
    }

    #[view(getFarmTokenId)]
    fn get_farm_token(&self, farm_address: &ManagedAddress) -> TokenIdentifier {
        let farm_token_id = self.farm_token_id().get_from_address(farm_address);
        self.require_valid_token_id(&farm_token_id);
        farm_token_id
    }

    #[view(getDivisionSafetyConstant)]
    fn get_division_safety_constant(&self, farm_address: &ManagedAddress) -> BigUint {
        let division_safety_constant = self
            .division_safety_constant()
            .get_from_address(farm_address);
        require!(division_safety_constant > 0, ERROR_DIVISION_CONSTANT_VALUE);
        division_safety_constant
    }

    #[storage_mapper("farm_token_id")]
    fn farm_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("farming_token_id")]
    fn farming_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("division_safety_constant")]
    fn division_safety_constant(&self) -> SingleValueMapper<BigUint>;

    #[view(getWrappedFarmTokenId)]
    #[storage_mapper("wrappedFarmTokenId")]
    fn wrapped_farm_token(&self) -> NonFungibleTokenMapper;

    #[view(getUnstakeFarmTokenId)]
    #[storage_mapper("unstakeFarmTokenId")]
    fn unstake_farm_token(&self) -> NonFungibleTokenMapper;

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
