use elrond_wasm::storage::mappers::StorageTokenWrapper;
use elrond_wasm::types::{EsdtLocalRole, ManagedAddress, MultiValueEncoded};

use elrond_wasm_debug::{
    managed_biguint, managed_token_id, rust_biguint, testing_framework::*, DebugApi,
};

use config::*;

use farm_staking::custom_rewards::CustomRewardsModule;

use farm_staking::*;
use farm_token::FarmTokenModule;
use pausable::{PausableModule, State};

pub static STAKING_FARM_TOKEN_ID: &[u8] = b"STAKEFARM-abcdef";
pub const DIVISION_SAFETY_CONSTANT: u64 = 1_000_000_000_000;
pub const MIN_UNBOND_EPOCHS: u64 = 5;
pub const MAX_APR: u64 = 2_500; // 25%
pub const PER_BLOCK_REWARD_AMOUNT: u64 = 5_000;
pub const TOTAL_REWARDS_AMOUNT: u64 = 1_000_000_000_000;

pub const USER_TOTAL_RIDE_TOKENS: u64 = 5_000_000_000;

pub fn setup_farm_staking<FarmObjBuilder>(
    b_mock: &mut BlockchainStateWrapper,
    farm_builder: FarmObjBuilder,
    farming_token_id: &[u8],
    reward_token_id: &[u8],
) -> ContractObjWrapper<farm_staking::ContractObj<DebugApi>, FarmObjBuilder>
where
    FarmObjBuilder: 'static + Copy + Fn() -> farm_staking::ContractObj<DebugApi>,
{
    let rust_zero = rust_biguint!(0u64);
    let owner_addr = b_mock.create_user_account(&rust_zero);
    let farm_wrapper =
        b_mock.create_sc_account(&rust_zero, Some(&owner_addr), farm_builder, "farm-staking");

    // init farm contract
    b_mock
        .execute_tx(&owner_addr, &farm_wrapper, &rust_zero, |sc| {
            sc.init(
                managed_token_id!(farming_token_id),
                managed_biguint!(DIVISION_SAFETY_CONSTANT),
                managed_biguint!(MAX_APR),
                MIN_UNBOND_EPOCHS,
                ManagedAddress::<DebugApi>::zero(),
                MultiValueEncoded::new(),
            );

            sc.farm_token()
                .set_token_id(managed_token_id!(STAKING_FARM_TOKEN_ID));

            sc.per_block_reward_amount()
                .set(&managed_biguint!(PER_BLOCK_REWARD_AMOUNT));

            sc.state().set(State::Active);
            sc.produce_rewards_enabled().set(true);
        })
        .assert_ok();

    b_mock.set_esdt_balance(&owner_addr, reward_token_id, &TOTAL_REWARDS_AMOUNT.into());
    b_mock
        .execute_esdt_transfer(
            &owner_addr,
            &farm_wrapper,
            reward_token_id,
            0,
            &TOTAL_REWARDS_AMOUNT.into(),
            |sc| {
                sc.top_up_rewards();
            },
        )
        .assert_ok();

    let farm_token_roles = [
        EsdtLocalRole::NftCreate,
        EsdtLocalRole::NftAddQuantity,
        EsdtLocalRole::NftBurn,
    ];
    b_mock.set_esdt_local_roles(
        farm_wrapper.address_ref(),
        STAKING_FARM_TOKEN_ID,
        &farm_token_roles[..],
    );

    let farming_token_roles = [EsdtLocalRole::Burn];
    b_mock.set_esdt_local_roles(
        farm_wrapper.address_ref(),
        farming_token_id,
        &farming_token_roles[..],
    );

    farm_wrapper
}
