multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use common_structs::{Epoch, Nonce};

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
