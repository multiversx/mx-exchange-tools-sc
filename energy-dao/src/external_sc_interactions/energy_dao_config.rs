multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use common_structs::{Epoch, Nonce};

use crate::common::{
    errors::{
        ERROR_DIVISION_CONSTANT_VALUE, ERROR_FARM_ALREADY_DEFINED, ERROR_FARM_DOES_NOT_EXIST,
        ERROR_FARM_HAS_FUNDS, ERROR_METASTAKING_ALREADY_DEFINED, ERROR_METASTAKING_DOES_NOT_EXIST,
        ERROR_METASTAKING_HAS_FUNDS,
    },
    rewards_wrapper::RewardsWrapper,
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

#[derive(TypeAbi, TopEncode, TopDecode, Debug)]
pub struct MetastakingState<M: ManagedTypeApi> {
    pub ms_staked_value: BigUint<M>,
    pub dual_yield_token_nonce: Nonce,
    pub lp_farm_reward_token_nonce: Nonce,
    pub lp_farm_reward_reserve: BigUint<M>,
    pub staking_reward_reserve: BigUint<M>,
    pub lp_farm_rps: BigUint<M>,
    pub staking_rps: BigUint<M>,
}

#[derive(TypeAbi, TopEncode, TopDecode, Debug, PartialEq)]
pub struct WrappedFarmTokenAttributes<M: ManagedTypeApi> {
    pub farm_address: ManagedAddress<M>,
    pub token_rps: BigUint<M>,
}

#[derive(TypeAbi, TopEncode, TopDecode, Debug, PartialEq)]
pub struct WrappedMetastakingTokenAttributes<M: ManagedTypeApi> {
    pub metastaking_address: ManagedAddress<M>,
    pub lp_farm_token_rps: BigUint<M>,
    pub staking_token_rps: BigUint<M>,
}

#[derive(TypeAbi, TopEncode, TopDecode, Debug, PartialEq)]
pub struct UnstakeTokenAttributes<M: ManagedTypeApi> {
    pub farm_address: ManagedAddress<M>,
    pub unstake_epoch: Epoch,
    pub token_nonce: Nonce,
}

#[derive(TypeAbi, TopEncode, TopDecode, Debug, PartialEq)]
pub struct UnstakeMetastakingAttributes<M: ManagedTypeApi> {
    pub metastaking_address: ManagedAddress<M>,
    pub unbond_token_id: TokenIdentifier<M>,
    pub unbond_token_nonce: Nonce,
}

pub const MAX_PERCENT: u64 = 10_000;

#[multiversx_sc::module]
pub trait EnergyDAOConfigModule: utils::UtilsModule {
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
    #[payable("EGLD")]
    #[endpoint(registerWrappedMetastakingToken)]
    fn register_wrapped_metastaking_token(
        &self,
        token_display_name: ManagedBuffer,
        token_ticker: ManagedBuffer,
        num_decimals: usize,
    ) {
        let payment_amount = self.call_value().egld_value();
        self.wrapped_metastaking_token().issue_and_set_all_roles(
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
    #[endpoint(registerUnstakeMetastakingToken)]
    fn register_unstake_metastaking_token(
        &self,
        token_display_name: ManagedBuffer,
        token_ticker: ManagedBuffer,
        num_decimals: usize,
    ) {
        let payment_amount = self.call_value().egld_value();
        self.unstake_metastaking_token().issue_and_set_all_roles(
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

    #[only_owner]
    #[endpoint(addMetastakingAddresses)]
    fn add_metastaking_addresses(&self, metastaking_addresses: MultiValueEncoded<ManagedAddress>) {
        for metastaking_address in metastaking_addresses {
            let metastaking_state_mapper = self.metastaking_state(&metastaking_address);
            require!(
                metastaking_state_mapper.is_empty(),
                ERROR_METASTAKING_ALREADY_DEFINED
            );
            self.require_sc_address(&metastaking_address);

            let metastaking_state = MetastakingState {
                ms_staked_value: BigUint::zero(),
                dual_yield_token_nonce: 0u64,
                lp_farm_reward_token_nonce: 0u64,
                lp_farm_reward_reserve: BigUint::zero(),
                staking_reward_reserve: BigUint::zero(),
                lp_farm_rps: BigUint::zero(),
                staking_rps: BigUint::zero(),
            };
            metastaking_state_mapper.set(metastaking_state);
        }
    }

    #[only_owner]
    #[endpoint(removeMetastakingAddresses)]
    fn remove_metastaking_addresses(
        &self,
        metastaking_addresses: MultiValueEncoded<ManagedAddress>,
    ) {
        for metastaking_address in metastaking_addresses {
            let metastaking_state_mapper = self.metastaking_state(&metastaking_address);
            require!(
                !metastaking_state_mapper.is_empty(),
                ERROR_METASTAKING_DOES_NOT_EXIST
            );
            let metastaking_state = metastaking_state_mapper.get();
            require!(
                metastaking_state.ms_staked_value == 0,
                ERROR_METASTAKING_HAS_FUNDS
            );
            metastaking_state_mapper.clear();
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

    #[view(getDualYieldTokenId)]
    fn get_dual_yield_token(&self, metastaking_address: &ManagedAddress) -> TokenIdentifier {
        let dual_yield_token_id = self
            .dual_yield_token_id()
            .get_from_address(metastaking_address);
        self.require_valid_token_id(&dual_yield_token_id);
        dual_yield_token_id
    }

    #[view(getLpFarmTokenId)]
    fn get_lp_farm_token(&self, metastaking_address: &ManagedAddress) -> TokenIdentifier {
        let lp_farm_token_id = self
            .lp_farm_token_id()
            .get_from_address(metastaking_address);
        self.require_valid_token_id(&lp_farm_token_id);
        lp_farm_token_id
    }

    #[view(getStakingTokenId)]
    fn get_staking_token(&self, metastaking_address: &ManagedAddress) -> TokenIdentifier {
        let staking_token_id = self
            .staking_token_id()
            .get_from_address(metastaking_address);
        self.require_valid_token_id(&staking_token_id);
        staking_token_id
    }

    #[view(getStakingFarmAddress)]
    fn get_staking_farm_address(&self, metastaking_address: &ManagedAddress) -> ManagedAddress {
        let staking_farm_address = self
            .staking_farm_address()
            .get_from_address(metastaking_address);
        self.require_sc_address(&staking_farm_address);
        staking_farm_address
    }

    #[storage_mapper("farm_token_id")]
    fn farm_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("farming_token_id")]
    fn farming_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("division_safety_constant")]
    fn division_safety_constant(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("dualYieldTokenId")]
    fn dual_yield_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("lpFarmTokenId")]
    fn lp_farm_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("stakingTokenId")]
    fn staking_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("stakingFarmAddress")]
    fn staking_farm_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getWrappedFarmTokenId)]
    #[storage_mapper("wrappedFarmTokenId")]
    fn wrapped_farm_token(&self) -> NonFungibleTokenMapper;

    #[view(getUnstakeFarmTokenId)]
    #[storage_mapper("unstakeFarmTokenId")]
    fn unstake_farm_token(&self) -> NonFungibleTokenMapper;

    #[view(getWrappedMetastakingTokenId)]
    #[storage_mapper("wrappedMetastakingTokenId")]
    fn wrapped_metastaking_token(&self) -> NonFungibleTokenMapper;

    #[view(getUnstakeMetastakingTokenId)]
    #[storage_mapper("unstakeMetastakingTokenId")]
    fn unstake_metastaking_token(&self) -> NonFungibleTokenMapper;

    #[view(getUnbondPeriod)]
    #[storage_mapper("unbondPeriod")]
    fn unbond_period(&self) -> SingleValueMapper<Epoch>;

    #[view(getExitPenaltyPercent)]
    #[storage_mapper("exitPenaltyPercent")]
    fn exit_penalty_percent(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("userExitFees")]
    fn user_exit_fees(&self) -> SingleValueMapper<RewardsWrapper<Self::Api>>;

    #[storage_mapper("farmState")]
    fn farm_state(&self, farm_address: &ManagedAddress) -> SingleValueMapper<FarmState<Self::Api>>;

    #[storage_mapper("metastakingState")]
    fn metastaking_state(
        &self,
        metastaking_address: &ManagedAddress,
    ) -> SingleValueMapper<MetastakingState<Self::Api>>;
}
