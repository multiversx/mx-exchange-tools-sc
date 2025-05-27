use core::convert::TryFrom;

use router::multi_pair_swap::{
    SWAP_TOKENS_FIXED_INPUT_FUNC_NAME, SWAP_TOKENS_FIXED_OUTPUT_FUNC_NAME,
};

use crate::{
    config::{self, MAX_PERCENTAGE, ROUTER_SWAP_ARGS_LEN, SEND_TOKENS_ARGS_LEN, SWAP_ARGS_LEN},
    external_sc_interactions,
};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub type PaymentsVec<M> = ManagedVec<M, EsdtTokenPayment<M>>;

#[derive(TypeAbi, TopEncode, TopDecode, PartialEq, ManagedVecItem)]
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
            let (task_type, mut args) = task.into_tuple();

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
                    self.smart_swap(payment_for_current_task, &mut payments_to_return, &mut args)
                }
                TaskType::SendEgldOrEsdt => {
                    require!(
                        args.len() == SEND_TOKENS_ARGS_LEN,
                        "Invalid number of arguments!"
                    );
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
            "EGLD can't be swapped!"
        );
        let payment_in = payment_for_current_task.unwrap_esdt();

        require!(
            args.len() == SWAP_ARGS_LEN,
            "Incorrect arguments for swap task!"
        );

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
            "Invalid function name for swap"
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
            let payment_out = returned_payments_by_pair.get(0);
            if returned_payments_by_pair.len() == 2 {
                let payment_in_leftover = returned_payments_by_pair.get(1);
                payments_to_return.push(payment_in_leftover);
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
            "EGLD can't be swapped!"
        );
        require!(
            args.len() % ROUTER_SWAP_ARGS_LEN == 0,
            "Invalid number of router swap arguments"
        );
        let payment_in = payment_for_current_task.unwrap_esdt();
        let mut returned_payments_by_router = self.multi_pair_swap(payment_in, args);

        require!(
            !returned_payments_by_router.is_empty(),
            "Router swap returned 0 payments"
        );

        let last_payment_index = returned_payments_by_router.len() - 1;
        let payment_out = returned_payments_by_router.take(last_payment_index);
        payments_to_return.append_vec(returned_payments_by_router);
        EgldOrEsdtTokenPayment::from(payment_out)
    }

    fn smart_swap(
        &self,
        payment_for_current_task: EgldOrEsdtTokenPayment,
        payments_to_return: &mut PaymentsVec<Self::Api>,
        args: &mut ManagedVec<ManagedBuffer>,
    ) -> EgldOrEsdtTokenPayment {
        require!(
            !payment_for_current_task.token_identifier.is_egld(),
            "EGLD can't be swapped!"
        );

        let mut payment_in = payment_for_current_task.unwrap_esdt();

        let args_cloned = args.clone();
        let mut aggregated_payment_out =
            self.get_empty_payment_out_from_smart_swap_args(args_cloned);

        let mut args_iter = args.into_iter();

        loop {
            let routes_no = match args_iter.next() {
                Some(count) => match count.parse_as_u64() {
                    Some(count) => count,
                    None => sc_panic!("Number of routes arguments is invalid"),
                },
                None => break,
            };

            // take the input amount for the swap
            let amount_in = match args_iter.next() {
                Some(amount) => BigUint::from(amount),
                None => break,
            };

            payment_in.amount = amount_in;

            // take args for a router_swap
            let router_args: ManagedVec<ManagedBuffer> = args_iter
                .clone()
                .take(ROUTER_SWAP_ARGS_LEN * routes_no as usize)
                .collect();

            let mut returned_payments_by_router =
                self.multi_pair_swap(payment_in.clone(), router_args);
            require!(
                !returned_payments_by_router.is_empty(),
                "Router swap returned 0 payments"
            );

            // aggregate all output_payments
            let partial_payment_out_index = returned_payments_by_router.len() - 1;
            let partial_payment_out = returned_payments_by_router.take(partial_payment_out_index);
            require!(
                partial_payment_out.token_identifier == aggregated_payment_out.token_identifier,
                "Router returned wrong payment output for smart swaps"
            );
            aggregated_payment_out.amount += partial_payment_out.amount;

            // concatenate all the other payments
            payments_to_return.append_vec(returned_payments_by_router);
        }

        let fee_percentage = self.smart_swap_fee_percentage().get();
        if fee_percentage != 0 {
            let fee_amount =
                self.calculate_fee_amount(&aggregated_payment_out.amount, fee_percentage);
            aggregated_payment_out.amount -= &fee_amount;
        }

        EgldOrEsdtTokenPayment::from(aggregated_payment_out)
    }

    fn get_empty_payment_out_from_smart_swap_args(
        &self,
        args: ManagedVec<ManagedBuffer>,
    ) -> EsdtTokenPayment<Self::Api> {
        let mut args_iter = args.into_iter();
        let routes_no = match args_iter.next() {
            Some(count) => count.parse_as_u64().unwrap(),
            None => sc_panic!("Smart Swap: Cannot retrieve number of routes"),
        };
        let _ = args_iter.next(); // this is the amount_in, we skip this arg

        // take args for a router_swap
        let router_args: ManagedVec<ManagedBuffer> = args_iter
            .clone()
            .take(ROUTER_SWAP_ARGS_LEN * routes_no as usize)
            .collect();

        let token_out_index = router_args.len() - 2; // token_out is the arg before last one
        let token_out = router_args.get(token_out_index);

        EsdtTokenPayment::new(token_out.clone_value().into(), 0, BigUint::zero())
    }

    fn calculate_fee_amount(&self, payment_amount: &BigUint, fee_percentage: u64) -> BigUint {
        payment_amount * fee_percentage / MAX_PERCENTAGE
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
            "The output token is less or different than the one required by user!"
        );
    }
}
