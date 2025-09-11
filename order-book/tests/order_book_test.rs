use crate::order_book_setup::OrderBookSetup;

pub mod order_book_setup;
pub mod pair_setup;
pub mod router_setup;

#[test]
fn setup_test() {
    let _ = OrderBookSetup::new(
        pair::contract_obj,
        router::contract_obj,
        order_book::contract_obj,
    );
}
