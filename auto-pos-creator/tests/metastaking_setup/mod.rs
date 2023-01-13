use std::{cell::RefCell, rc::Rc};

use elrond_wasm::{
    storage::mappers::StorageTokenWrapper,
    types::{Address, EsdtLocalRole},
};
use elrond_wasm_debug::{
    managed_address, managed_token_id, rust_biguint,
    testing_framework::{BlockchainStateWrapper, ContractObjWrapper},
    DebugApi,
};

use farm_staking_proxy::{dual_yield_token::DualYieldTokenModule, *};

pub static DUAL_YIELD_TOKEN_ID: &[u8] = b"DUALYIELD-123456";

pub struct MetastakingSetup<MetastakingObjBuilder>
where
    MetastakingObjBuilder: 'static + Copy + Fn() -> farm_staking_proxy::ContractObj<DebugApi>,
{
    pub b_mock: Rc<RefCell<BlockchainStateWrapper>>,
    pub ms_wrapper:
        ContractObjWrapper<farm_staking_proxy::ContractObj<DebugApi>, MetastakingObjBuilder>,
}

#[allow(clippy::too_many_arguments)]
pub fn setup_metastaking<MetastakingObjBuilder>(
    b_mock: &mut BlockchainStateWrapper,
    ms_builder: MetastakingObjBuilder,
    owner: &Address,
    lp_farm_address: &Address,
    staking_farm_address: &Address,
    pair_address: &Address,
    staking_token_id: &[u8],
    lp_farm_token_id: &[u8],
    staking_farm_token_id: &[u8],
    lp_token_id: &[u8],
) -> ContractObjWrapper<farm_staking_proxy::ContractObj<DebugApi>, MetastakingObjBuilder>
where
    MetastakingObjBuilder: 'static + Copy + Fn() -> farm_staking_proxy::ContractObj<DebugApi>,
{
    let rust_zero = rust_biguint!(0u64);
    let proxy_wrapper =
        b_mock.create_sc_account(&rust_zero, Some(owner), ms_builder, "metastaking");

    b_mock
        .execute_tx(owner, &proxy_wrapper, &rust_zero, |sc| {
            sc.init(
                managed_address!(lp_farm_address),
                managed_address!(staking_farm_address),
                managed_address!(pair_address),
                managed_token_id!(staking_token_id),
                managed_token_id!(lp_farm_token_id),
                managed_token_id!(staking_farm_token_id),
                managed_token_id!(lp_token_id),
            );

            sc.dual_yield_token()
                .set_token_id(managed_token_id!(DUAL_YIELD_TOKEN_ID));
        })
        .assert_ok();

    let dual_yield_token_roles = [
        EsdtLocalRole::NftCreate,
        EsdtLocalRole::NftAddQuantity,
        EsdtLocalRole::NftBurn,
    ];
    b_mock.set_esdt_local_roles(
        proxy_wrapper.address_ref(),
        DUAL_YIELD_TOKEN_ID,
        &dual_yield_token_roles[..],
    );

    proxy_wrapper
}
