use std::cell::RefCell;
use std::rc::Rc;

use multiversx_sc::types::{Address, EsdtLocalRole, ManagedAddress, MultiValueEncoded};
use multiversx_sc_scenario::whitebox::TxTokenTransfer;
use multiversx_sc_scenario::{
    managed_address, managed_biguint, managed_token_id, rust_biguint, whitebox::*,
    DebugApi,
};

use pair::config::ConfigModule;
use pair::safe_price::SafePriceModule;
use pair::*;
use pausable::{PausableModule, State};

pub struct PairSetup<PairObjBuilder>
where
    PairObjBuilder: 'static + Copy + Fn() -> pair::ContractObj<DebugApi>,
{
    pub b_mock: Rc<RefCell<BlockchainStateWrapper>>,
    pub first_token_id: Vec<u8>,
    pub second_token_id: Vec<u8>,
    pub lp_token_id: Vec<u8>,
    pub pair_wrapper: ContractObjWrapper<pair::ContractObj<DebugApi>, PairObjBuilder>,
}

impl<PairObjBuilder> PairSetup<PairObjBuilder>
where
    PairObjBuilder: 'static + Copy + Fn() -> pair::ContractObj<DebugApi>,
{
    pub fn new(
        b_mock: Rc<RefCell<BlockchainStateWrapper>>,
        pair_builder: PairObjBuilder,
        owner: &Address,
        first_token_id: &[u8],
        second_token_id: &[u8],
        lp_token_id: &[u8],
    ) -> Self {
        let rust_zero = rust_biguint!(0u64);
        let pair_wrapper =
            b_mock
                .borrow_mut()
                .create_sc_account(&rust_zero, Some(owner), pair_builder, "pair");

        b_mock
            .borrow_mut()
            .execute_tx(owner, &pair_wrapper, &rust_zero, |sc| {
                sc.init(
                    managed_token_id!(first_token_id),
                    managed_token_id!(second_token_id),
                    managed_address!(owner),
                    managed_address!(owner),
                    0,
                    0,
                    ManagedAddress::<DebugApi>::zero(),
                    MultiValueEncoded::<DebugApi, ManagedAddress<DebugApi>>::new(),
                );

                sc.lp_token_identifier()
                    .set(&managed_token_id!(lp_token_id));
                sc.state().set(State::Active);
                sc.set_max_observations_per_record(10);
            })
            .assert_ok();

        let lp_token_roles = [EsdtLocalRole::Mint, EsdtLocalRole::Burn];
        b_mock.borrow_mut().set_esdt_local_roles(
            pair_wrapper.address_ref(),
            lp_token_id,
            &lp_token_roles[..],
        );

        PairSetup {
            b_mock,
            first_token_id: first_token_id.to_vec(),
            second_token_id: second_token_id.to_vec(),
            lp_token_id: lp_token_id.to_vec(),
            pair_wrapper,
        }
    }

    pub fn add_liquidity(
        &mut self,
        caller: &Address,
        first_token_amount: u64,
        second_token_amount: u64,
    ) {
        let payments = vec![
            TxTokenTransfer {
                token_identifier: self.first_token_id.clone(),
                nonce: 0,
                value: rust_biguint!(first_token_amount),
            },
            TxTokenTransfer {
                token_identifier: self.second_token_id.clone(),
                nonce: 0,
                value: rust_biguint!(second_token_amount),
            },
        ];

        self.b_mock
            .borrow_mut()
            .execute_esdt_multi_transfer(caller, &self.pair_wrapper, &payments, |sc| {
                _ = sc.add_liquidity(managed_biguint!(1), managed_biguint!(1));
            })
            .assert_ok();
    }

    pub fn swap_fixed_input(
        &mut self,
        caller: &Address,
        input_token_id: &[u8],
        input_token_amount: u64,
    ) -> (Vec<u8>, num_bigint::BigUint) {
        let out_token_id = if input_token_id == self.first_token_id {
            self.second_token_id.clone()
        } else {
            self.first_token_id.clone()
        };
        let mut amount_out = rust_biguint!(0);

        self.b_mock
            .borrow_mut()
            .execute_esdt_transfer(
                caller,
                &self.pair_wrapper,
                input_token_id,
                0,
                &rust_biguint!(input_token_amount),
                |sc| {
                    let out_payment = sc.swap_tokens_fixed_input(
                        managed_token_id!(&out_token_id[..]),
                        managed_biguint!(1),
                    );
                    amount_out = num_bigint::BigUint::from_bytes_be(
                        out_payment.amount.to_bytes_be().as_slice(),
                    );
                },
            )
            .assert_ok();

        (out_token_id, amount_out)
    }
}
