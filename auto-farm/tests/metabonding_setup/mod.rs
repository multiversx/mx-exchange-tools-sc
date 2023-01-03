use elrond_wasm::{elrond_codec::multi_types::OptionalValue, types::Address};
use elrond_wasm_debug::{
    managed_address, managed_biguint, managed_buffer, managed_token_id, rust_biguint,
    testing_framework::{BlockchainStateWrapper, ContractObjWrapper},
    DebugApi,
};

use elrond_wasm_modules::pause::PauseModule;
use metabonding::{project::ProjectModule, rewards::RewardsModule, Metabonding};

// associated private key - used for generating the signatures (please don't steal my funds)
// 3eb200ef228e593d49a522f92587889fedfc091629d175873b64ca0ab3b4514d52773868c13654355cca16adb389b09201fabf5d9d4b795ebbdae5b361b46f20
pub static SIGNER_ADDRESS: [u8; 32] =
    hex_literal::hex!("52773868c13654355cca16adb389b09201fabf5d9d4b795ebbdae5b361b46f20");
pub static FIRST_PROJ_ID: &[u8] = b"FirstProj";
pub static SECOND_PROJ_ID: &[u8] = b"SecondProj";
pub static FIRST_PROJ_TOKEN: &[u8] = b"PROJ-123456";
pub static SECOND_PROJ_TOKEN: &[u8] = b"COOL-123456";
pub const TOTAL_FIRST_PROJ_TOKENS: u64 = 1_000_000_000;
pub const TOTAL_SECOND_PROJ_TOKENS: u64 = 2_000_000_000;

pub fn setup_metabonding<MetabondingObjBuilder>(
    b_mock: &mut BlockchainStateWrapper,
    metabonding_builder: MetabondingObjBuilder,
) -> ContractObjWrapper<metabonding::ContractObj<DebugApi>, MetabondingObjBuilder>
where
    MetabondingObjBuilder: 'static + Copy + Fn() -> metabonding::ContractObj<DebugApi>,
{
    let rust_zero = rust_biguint!(0);
    let owner_addr = b_mock.create_user_account(&rust_zero);
    let first_project_owner = b_mock.create_user_account(&rust_zero);
    let second_project_owner = b_mock.create_user_account(&rust_zero);

    // need to create some fixed addresses to reuse the signatures from mandos
    // address:user1 from mandos
    let first_user_addr = Address::from(hex_literal::hex!(
        "75736572315F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F"
    ));
    b_mock.create_user_account_fixed_address(&first_user_addr, &rust_zero);

    // address:user2 from mandos
    let second_user_addr = Address::from(hex_literal::hex!(
        "75736572325F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F"
    ));
    b_mock.create_user_account_fixed_address(&second_user_addr, &rust_zero);

    b_mock.set_esdt_balance(
        &first_project_owner,
        FIRST_PROJ_TOKEN,
        &rust_biguint!(TOTAL_FIRST_PROJ_TOKENS),
    );
    b_mock.set_esdt_balance(
        &second_project_owner,
        SECOND_PROJ_TOKEN,
        &rust_biguint!((TOTAL_SECOND_PROJ_TOKENS)),
    );

    let current_epoch = 5;
    b_mock.set_block_epoch(current_epoch);

    let mb_wrapper = b_mock.create_sc_account(
        &rust_zero,
        Some(&owner_addr),
        metabonding_builder,
        "metabonding wasm path",
    );
    b_mock
        .execute_tx(&owner_addr, &mb_wrapper, &rust_zero, |sc| {
            let signer_addr = managed_address!(&Address::from(&SIGNER_ADDRESS));
            sc.init(
                signer_addr.clone(),
                OptionalValue::None,
                OptionalValue::None,
            );
        })
        .assert_ok();

    b_mock
        .execute_tx(&owner_addr, &mb_wrapper, &rust_zero, |sc| {
            sc.add_project(
                managed_buffer!(FIRST_PROJ_ID),
                managed_address!(&first_project_owner),
                managed_token_id!(FIRST_PROJ_TOKEN),
                managed_biguint!(TOTAL_FIRST_PROJ_TOKENS),
                1,
                3,
                0,
            );

            sc.add_project(
                managed_buffer!(SECOND_PROJ_ID),
                managed_address!(&second_project_owner),
                managed_token_id!(SECOND_PROJ_TOKEN),
                managed_biguint!(TOTAL_SECOND_PROJ_TOKENS),
                2,
                5,
                0,
            );
        })
        .assert_ok();

    b_mock
        .execute_esdt_transfer(
            &first_project_owner,
            &mb_wrapper,
            FIRST_PROJ_TOKEN,
            0,
            &rust_biguint!(TOTAL_FIRST_PROJ_TOKENS),
            |sc| {
                sc.deposit_rewards(managed_buffer!(FIRST_PROJ_ID));
            },
        )
        .assert_ok();

    b_mock
        .execute_esdt_transfer(
            &second_project_owner,
            &mb_wrapper,
            SECOND_PROJ_TOKEN,
            0,
            &rust_biguint!(TOTAL_SECOND_PROJ_TOKENS),
            |sc| {
                sc.deposit_rewards(managed_buffer!(SECOND_PROJ_ID));
            },
        )
        .assert_ok();

    b_mock.set_block_epoch(20);

    b_mock
        .execute_tx(&owner_addr, &mb_wrapper, &rust_zero, |sc| {
            sc.add_rewards_checkpoint(1, managed_biguint!(100_000), managed_biguint!(0));
        })
        .assert_ok();

    b_mock
        .execute_tx(&owner_addr, &mb_wrapper, &rust_zero, |sc| {
            sc.add_rewards_checkpoint(2, managed_biguint!(200_000), managed_biguint!(0));
        })
        .assert_ok();

    b_mock
        .execute_tx(&owner_addr, &mb_wrapper, &rust_zero, |sc| {
            sc.unpause_endpoint();
        })
        .assert_ok();

    mb_wrapper
}
