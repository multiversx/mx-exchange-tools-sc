multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use common_structs::Epoch;

use crate::common::{
    errors::{
        ERROR_DIVISION_CONSTANT_VALUE, ERROR_FARM_ALREADY_DEFINED, ERROR_FARM_DOES_NOT_EXIST,
        ERROR_FARM_HAS_FUNDS, ERROR_METASTAKING_ALREADY_DEFINED, ERROR_METASTAKING_DOES_NOT_EXIST,
        ERROR_METASTAKING_HAS_FUNDS, ERROR_PERCENTAGE_VALUE,
    },
    rewards_wrapper::RewardsWrapper,
    structs::{FarmState, MetastakingState},
};

pub const MAX_PERCENTAGE: u64 = 10_000; // 100.00%

#[multiversx_sc::module]
pub trait EnergyDAOConfigModule:
    utils::UtilsModule + permissions_module::PermissionsModule
{
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
    #[endpoint(setExitPenaltyPercent)]
    fn set_exit_penalty_percent(&self, exit_penalty_percent: u64) {
        require!(
            exit_penalty_percent <= MAX_PERCENTAGE,
            ERROR_PERCENTAGE_VALUE
        );
        self.exit_penalty_percent().set(exit_penalty_percent);
    }

    /// Endpoint that allows the owner or a trustworthy admin address to add a new farm
    #[endpoint(addFarms)]
    fn add_farms(&self, farms: MultiValueEncoded<ManagedAddress>) {
        self.require_caller_has_owner_permissions();
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

    /// Endpoint that allows the owner or a trustworthy admin address to remove a farm, if no funds were deposited
    /// It can be updated to have a more enforcing approach, by properly sending the funds back to the users, before removing the farm
    #[endpoint(removeFarms)]
    fn remove_farms(&self, farms: MultiValueEncoded<ManagedAddress>) {
        self.require_caller_has_owner_permissions();
        for farm in farms {
            let farm_state_mapper = self.farm_state(&farm);
            require!(!farm_state_mapper.is_empty(), ERROR_FARM_DOES_NOT_EXIST);
            let farm_state = farm_state_mapper.get();
            require!(farm_state.farm_staked_value == 0, ERROR_FARM_HAS_FUNDS);
            farm_state_mapper.clear();
        }
    }

    #[endpoint(addMetastakingAddresses)]
    fn add_metastaking_addresses(&self, metastaking_addresses: MultiValueEncoded<ManagedAddress>) {
        self.require_caller_has_owner_permissions();
        for metastaking_address in metastaking_addresses {
            let metastaking_state_mapper = self.metastaking_state(&metastaking_address);
            require!(
                metastaking_state_mapper.is_empty(),
                ERROR_METASTAKING_ALREADY_DEFINED
            );
            self.require_sc_address(&metastaking_address);

            let lp_farm_address = self.get_lp_farm_address(&metastaking_address);
            self.require_sc_address(&lp_farm_address);
            self.lp_farm_metastaking_address(&lp_farm_address)
                .set(metastaking_address);

            let metastaking_state = MetastakingState {
                metastaking_token_supply: BigUint::zero(),
                dual_yield_amount: BigUint::zero(),
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

    #[endpoint(removeMetastakingAddresses)]
    fn remove_metastaking_addresses(
        &self,
        metastaking_addresses: MultiValueEncoded<ManagedAddress>,
    ) {
        self.require_caller_has_owner_permissions();
        for metastaking_address in metastaking_addresses {
            let metastaking_state_mapper = self.metastaking_state(&metastaking_address);
            require!(
                !metastaking_state_mapper.is_empty(),
                ERROR_METASTAKING_DOES_NOT_EXIST
            );
            let metastaking_state = metastaking_state_mapper.get();
            require!(
                metastaking_state.metastaking_token_supply == 0,
                ERROR_METASTAKING_HAS_FUNDS
            );

            let lp_farm_address = self.get_lp_farm_address(&metastaking_address);
            self.require_sc_address(&lp_farm_address);
            self.lp_farm_metastaking_address(&lp_farm_address).clear();

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

    /// We use the division_safety_constant mapper as a way to access the value from each particular farm
    /// Because we do not actually save in the storage a certain value for each farm
    /// We should not have any performance penalty, as we read from the storage only once
    #[view(getDivisionSafetyConstant)]
    fn get_division_safety_constant(&self, farm_address: &ManagedAddress) -> BigUint {
        let division_safety_constant = self
            .division_safety_constant()
            .get_from_address(farm_address);
        require!(division_safety_constant > 0, ERROR_DIVISION_CONSTANT_VALUE);
        division_safety_constant
    }

    // We use the minimum_farming_epochs variable from the Farm SC, to have a clear alignment with the farm penalty period
    #[view(getMinimumFarmingEpoch)]
    fn get_minimum_farming_epochs(&self, farm_address: &ManagedAddress) -> Epoch {
        self.minimum_farming_epochs().get_from_address(farm_address)
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

    #[view(getLpFarmAddress)]
    fn get_lp_farm_address(&self, metastaking_address: &ManagedAddress) -> ManagedAddress {
        let lp_farm_address = self.lp_farm_address().get_from_address(metastaking_address);
        self.require_sc_address(&lp_farm_address);
        lp_farm_address
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

    #[storage_mapper("minimum_farming_epochs")]
    fn minimum_farming_epochs(&self) -> SingleValueMapper<Epoch>;

    #[storage_mapper("dualYieldTokenId")]
    fn dual_yield_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("lpFarmTokenId")]
    fn lp_farm_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("stakingTokenId")]
    fn staking_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("lpFarmAddress")]
    fn lp_farm_address(&self) -> SingleValueMapper<ManagedAddress>;

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

    #[storage_mapper("lpFarmMetastakingAddress")]
    fn lp_farm_metastaking_address(
        &self,
        farm_address: &ManagedAddress,
    ) -> SingleValueMapper<ManagedAddress>;
}
