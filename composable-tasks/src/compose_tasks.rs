use core::convert::TryFrom;

use pair::errors::ERROR_INVALID_ARGS;
use router::multi_pair_swap::{
    SWAP_TOKENS_FIXED_INPUT_FUNC_NAME, SWAP_TOKENS_FIXED_OUTPUT_FUNC_NAME,
};

use crate::{
    config::{self, ROUTER_SWAP_ARGS_LEN, SEND_TOKENS_ARGS_LEN, SWAP_ARGS_LEN},
    errors::*,
    events, external_sc_interactions, task_types,
};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub type PaymentsVec<M> = ManagedVec<M, EsdtTokenPayment<M>>;

#[type_abi]
#[derive(TopEncode, TopDecode, PartialEq, ManagedVecItem)]
pub enum TaskType {
    WrapEGLD,
    UnwrapEGLD,
    Swap,
    RouterSwap,
    SendEgldOrEsdt,
    SmartSwap,
}

#[multiversx_sc::module]
pub trait TaskCall:
    external_sc_interactions::pair_actions::PairActionsModule
    + external_sc_interactions::router_actions::RouterActionsModule
    + external_sc_interactions::wegld_swap::WegldWrapModule
    + config::ConfigModule
    + events::EventsModule
    + task_types::smart_swap::SmartSwapModule
{
    #[payable("*")]
    #[endpoint(composeTasks)]
    fn compose_tasks(
        &self,
        min_expected_token_out: EgldOrEsdtTokenPayment,
        tasks: MultiValueEncoded<MultiValue2<TaskType, ManagedVec<ManagedBuffer>>>,
    ) {
        let mut payment_for_next_task = self.call_value().egld_or_single_esdt();
        let mut payments_to_return = PaymentsVec::new();

        let mut dest_addr = self.blockchain().get_caller();

        for task in tasks.into_iter() {
            let (task_type, args) = task.into_tuple();

            let payment_for_current_task = payment_for_next_task.clone();

            payment_for_next_task = match task_type {
                TaskType::WrapEGLD => self.wrap_egld(payment_for_current_task),
                TaskType::UnwrapEGLD => self.unwrap_egld(payment_for_current_task),
                TaskType::Swap => {
                    self.swap(payment_for_current_task, &mut payments_to_return, args)
                }
                TaskType::RouterSwap => {
                    self.router_swap(payment_for_current_task, &mut payments_to_return, args)
                }
                TaskType::SmartSwap => {
                    self.smart_swap(payment_for_current_task, &mut payments_to_return, args)
                }
                TaskType::SendEgldOrEsdt => {
                    require!(args.len() == SEND_TOKENS_ARGS_LEN, ERROR_INVALID_ARGS);
                    let new_destination = ManagedAddress::try_from(args.get(0).clone_value())
                        .unwrap_or_else(|err| sc_panic!(err));

                    dest_addr = new_destination;
                    break;
                }
            };
        }
        self.send_resulted_payments(
            dest_addr,
            min_expected_token_out,
            payment_for_next_task,
            &mut payments_to_return,
        )
    }

    fn swap(
        &self,
        payment_for_current_task: EgldOrEsdtTokenPayment,
        payments_to_return: &mut PaymentsVec<Self::Api>,
        args: ManagedVec<ManagedBuffer>,
    ) -> EgldOrEsdtTokenPayment {
        require!(
            !payment_for_current_task.token_identifier.is_egld(),
            ERROR_CANNOT_SWAP_EGLD
        );
        let payment_in = payment_for_current_task.unwrap_esdt();

        require!(args.len() == SWAP_ARGS_LEN, ERROR_INCORRECT_ARGS);

        let function_in_out = args.get(0).clone_value();
        let token_out = TokenIdentifier::from(args.get(1).clone_value());
        let min_amount_out = BigUint::from(args.get(2).clone_value());

        // if function_in_out
        let swap_tokens_fixed_input_function =
            ManagedBuffer::from(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME);
        let swap_tokens_fixed_output_function =
            ManagedBuffer::from(SWAP_TOKENS_FIXED_OUTPUT_FUNC_NAME);
        require!(
            function_in_out == swap_tokens_fixed_input_function
                || function_in_out == swap_tokens_fixed_output_function,
            ERROR_INVALID_FUNCTION_NAME
        );

        let payment_out = if function_in_out == swap_tokens_fixed_input_function {
            self.perform_swap_tokens_fixed_input(
                payment_in.token_identifier,
                payment_in.amount,
                token_out,
                min_amount_out,
            )
        } else {
            let returned_payments_by_pair = self.perform_swap_tokens_fixed_output(
                payment_in.token_identifier,
                payment_in.amount,
                token_out,
                min_amount_out,
            );
            let payment_out = returned_payments_by_pair.get(0).clone();
            if returned_payments_by_pair.len() == 2 {
                let payment_in_leftover = returned_payments_by_pair.get(1);
                payments_to_return.push(payment_in_leftover.clone());
            }
            payment_out
        };

        payment_out.into()
    }

    fn router_swap(
        &self,
        payment_for_current_task: EgldOrEsdtTokenPayment,
        payments_to_return: &mut PaymentsVec<Self::Api>,
        args: ManagedVec<ManagedBuffer>,
    ) -> EgldOrEsdtTokenPayment {
        require!(
            !payment_for_current_task.token_identifier.is_egld(),
            ERROR_CANNOT_SWAP_EGLD
        );
        require!(
            args.len() % ROUTER_SWAP_ARGS_LEN == 0,
            ERROR_INVALID_NUMBER_ROUTER_SWAP_ARGS
        );
        let payment_in = payment_for_current_task.unwrap_esdt();
        let mut returned_payments_by_router = self.multi_pair_swap(payment_in, args);

        require!(
            !returned_payments_by_router.is_empty(),
            ERROR_ROUTER_SWAP_0_PAYMENTS
        );

        let last_payment_index = returned_payments_by_router.len() - 1;
        let payment_out = returned_payments_by_router.take(last_payment_index);
        payments_to_return.append_vec(returned_payments_by_router);
        EgldOrEsdtTokenPayment::from(payment_out)
    }

    fn send_resulted_payments(
        &self,
        dest_addr: ManagedAddress,
        min_expected_token_out: EgldOrEsdtTokenPayment,
        payment_for_current_task: EgldOrEsdtTokenPayment,
        payments_to_return: &mut PaymentsVec<Self::Api>,
    ) {
        self.require_min_expected_token(&min_expected_token_out, &payment_for_current_task);
        if payment_for_current_task.token_identifier.is_egld() {
            self.send()
                .direct_egld(&dest_addr, &payment_for_current_task.amount);
        } else {
            payments_to_return.push(EsdtTokenPayment::new(
                payment_for_current_task.token_identifier.unwrap_esdt(),
                payment_for_current_task.token_nonce,
                payment_for_current_task.amount,
            ));
        }
        if !payments_to_return.is_empty() {
            self.send().direct_multi(&dest_addr, payments_to_return);
        }
    }

    fn require_min_expected_token(
        &self,
        expected_token: &EgldOrEsdtTokenPayment,
        token_out: &EgldOrEsdtTokenPayment,
    ) {
        require!(
            expected_token.token_identifier == token_out.token_identifier
                && expected_token.amount <= token_out.amount,
            ERROR_WRONG_RETURNED_TOKEN_IDENTIFIER
        );
    }
}
