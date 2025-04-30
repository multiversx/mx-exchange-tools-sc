use core::convert::TryFrom;

use router::multi_pair_swap::{
    SWAP_TOKENS_FIXED_INPUT_FUNC_NAME, SWAP_TOKENS_FIXED_OUTPUT_FUNC_NAME,
};

use crate::{
    config::{ROUTER_SWAP_ARGS_LEN, SEND_TOKENS_ARGS_LEN, SMART_SWAP_ARGS_LEN, SWAP_ARGS_LEN},
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
    SmartSwap,
    SendEgldOrEsdt,
}

#[multiversx_sc::module]
pub trait TaskCall:
    external_sc_interactions::pair_actions::PairActionsModule
    + external_sc_interactions::router_actions::RouterActionsModule
    + external_sc_interactions::wegld_swap::WegldWrapModule
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

        let mut smart_swap_input_payment: EsdtTokenPayment;
        let mut smart_swap_output_payment: EsdtTokenPayment;

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
                    let smart_swap_input_payment_mapper = self.smart_swap_input_payment();

                    let input_payment_for_smart_swap =
                        if smart_swap_input_payment.amount == BigUint::zero() {
                            // First smart swap
                            payment_for_current_task
                        } else {
                            // Continuation of smart swap
                            smart_swap_input_payment_mapper.get()
                        };

                    let mut output_payment_from_smart_swap = self.smart_swap(
                        input_payment_for_smart_swap,
                        &mut payments_to_return,
                        &mut args,
                    );

                    // Update Smart Swap output payment
                    let smart_swap_output_payment_mapper = self.smart_swap_output_payment();
                    if smart_swap_output_payment_mapper.is_empty() {
                        smart_swap_output_payment_mapper
                            .set(output_payment_from_smart_swap.clone());
                    } else {
                        let smart_swap_output_payment = smart_swap_output_payment_mapper.get();
                        output_payment_from_smart_swap.amount += smart_swap_output_payment.amount;

                        smart_swap_output_payment_mapper
                            .set(output_payment_from_smart_swap.clone());
                    }

                    output_payment_from_smart_swap
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
        require!(
            args.len() % SMART_SWAP_ARGS_LEN == 0,
            "Invalid number of smart swap arguments"
        );
        let mut payment_in = payment_for_current_task.unwrap_esdt();

        //////// Varianta 2: Get payment input amount from first argument
        // require!(args.len() > 0, "Invalid arguments for smart swap");
        // let amount_in = BigUint::from(args.take(0));
        // payment_in.amount = amount_in;

        // let mut returned_payments_by_router = self.multi_pair_swap(payment_in, args.clone());

        // require!(
        //     !returned_payments_by_router.is_empty(),
        //     "Smart Swap: router swap returned 0 payments"
        // );

        // let last_payment_index = returned_payments_by_router.len() - 1;
        // let payment_out = returned_payments_by_router.take(last_payment_index);
        // payments_to_return.append_vec(returned_payments_by_router);
        // EgldOrEsdtTokenPayment::from(payment_out)

        let args_cloned = args.clone();
        let mut aggregated_payment_out =
            self.get_empty_payment_out_from_smart_swap_args(args_cloned);

        let mut args_iter = args.into_iter();

        loop {
            let router_counter = match args_iter.next() {
                Some(count) => count.parse_as_u64().unwrap(),
                None => sc_panic!("TODO"),
            };

            // take the input amount for the swap
            let amount_in = match args_iter.next() {
                Some(amount) => BigUint::from(amount),
                None => break,
            };

            let payment_in = payment_for_current_task;
            payment_in.amount = amount_in;

            // take args for a router_swap
            args_iter.
            let router_args: ManagedVec<ManagedBuffer> =
                args_iter.by_ref().take(ROUTER_SWAP_ARGS_LEN).collect();

            let mut returned_payments_by_router = self.multi_pair_swap(payment_in, router_args);
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

        EgldOrEsdtTokenPayment::from(aggregated_payment_out)
    }

    fn get_empty_payment_out_from_smart_swap_args(
        &self,
        args: ManagedVec<ManagedBuffer>,
    ) -> EsdtTokenPayment<Self::Api> {
        let mut args_iter = args.into_iter();
        let token_out = match args_iter.next() {
            Some(token_id) => TokenIdentifier::from(token_id),
            None => sc_panic!("Could not retrieve output payment from smart swaps args"),
        };

        EsdtTokenPayment::new(token_out, 0, BigUint::zero())
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

    #[storage_mapper("smartSwapInputPayment")]
    fn smart_swap_input_payment(&self) -> SingleValueMapper<EgldOrEsdtTokenPayment<Self::Api>>;

    #[storage_mapper("smartSwapOutputPayment")]
    fn smart_swap_output_payment(&self) -> SingleValueMapper<EgldOrEsdtTokenPayment<Self::Api>>;
}
