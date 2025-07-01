use crate::dca_module_setup::DcaModuleSetup;

pub mod dca_module_setup;

// stolen from pos creator
pub mod pair_setup;
pub mod router_setup;

#[test]
fn setup_test() {
    let _ = DcaModuleSetup::new(
        pair::contract_obj,
        router::contract_obj,
        dca_module::contract_obj,
    );
}
