use auto_farm::common::common_storage::CommonStorageModule;
use auto_farm::registration::RegistrationModule;
use auto_farm::AutoFarm;
use elrond_wasm_debug::{managed_address, rust_biguint};
use tests_common::farm_with_locked_rewards_setup::FarmSetup;

const FEE_PERCENTAGE: u64 = 1_000; // 10%

#[test]
fn register_test() {
    let rust_zero = rust_biguint!(0);
    let farm_setup = FarmSetup::new(
        farm_with_locked_rewards::contract_obj,
        energy_factory::contract_obj,
    );

    let first_user = farm_setup.first_user.clone();
    let energy_factory_addr = farm_setup.energy_factory_wrapper.address_ref().clone();

    let proxy_address = farm_setup
        .b_mock
        .borrow_mut()
        .create_user_account(&rust_zero);
    let auto_farm_wrapper = farm_setup.b_mock.borrow_mut().create_sc_account(
        &rust_zero,
        Some(&farm_setup.owner),
        auto_farm::contract_obj,
        "auto farm",
    );

    farm_setup
        .b_mock
        .borrow_mut()
        .execute_tx(&farm_setup.owner, &auto_farm_wrapper, &rust_zero, |sc| {
            sc.init(
                managed_address!(&proxy_address),
                FEE_PERCENTAGE,
                managed_address!(&energy_factory_addr),
                managed_address!(&energy_factory_addr), // unused here
                managed_address!(&energy_factory_addr), // unused here
            );
        })
        .assert_ok();

    // register ok
    farm_setup
        .b_mock
        .borrow_mut()
        .execute_tx(&first_user, &auto_farm_wrapper, &rust_zero, |sc| {
            sc.register();

            assert_eq!(sc.user_ids().get_id(&managed_address!(&first_user)), 1);
        })
        .assert_ok();

    // try register again
    farm_setup
        .b_mock
        .borrow_mut()
        .execute_tx(&first_user, &auto_farm_wrapper, &rust_zero, |sc| {
            sc.register();
        })
        .assert_user_error("Address already registered");

    // unregister
    farm_setup
        .b_mock
        .borrow_mut()
        .execute_tx(&first_user, &auto_farm_wrapper, &rust_zero, |sc| {
            let _ = sc.withdraw_all_and_unregister();

            assert_eq!(sc.user_ids().get_id(&managed_address!(&first_user)), 0);
        })
        .assert_ok();

    // try unregister again
    farm_setup
        .b_mock
        .borrow_mut()
        .execute_tx(&first_user, &auto_farm_wrapper, &rust_zero, |sc| {
            let _ = sc.withdraw_all_and_unregister();
        })
        .assert_user_error("Unknown address");

    // register again - ok
    farm_setup
        .b_mock
        .borrow_mut()
        .execute_tx(&first_user, &auto_farm_wrapper, &rust_zero, |sc| {
            sc.register();

            assert_eq!(sc.user_ids().get_id(&managed_address!(&first_user)), 2);
        })
        .assert_ok();
}
