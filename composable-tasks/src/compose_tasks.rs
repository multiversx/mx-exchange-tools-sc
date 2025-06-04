use core::convert::TryFrom;

use pair::errors::ERROR_INVALID_ARGS;
use router::multi_pair_swap::{
    SWAP_TOKENS_FIXED_INPUT_FUNC_NAME, SWAP_TOKENS_FIXED_OUTPUT_FUNC_NAME,
};

use crate::{
    config::{
        self, MAX_PERCENTAGE, ROUTER_SWAP_ARGS_LEN, ROUTER_TOKEN_OUT_FROM_END_OFFSET,
        SEND_TOKENS_ARGS_LEN, SMART_SWAP_MIN_ARGS_LEN, SWAP_ARGS_LEN,
    },
    errors::{
        ERROR_ACC_AMOUNT_EXCEEDS_PAYMENT_IN, ERROR_CANNOT_SWAP_EGLD, ERROR_INCORRECT_ARGS,
        ERROR_INVALID_FUNCTION_NAME, ERROR_INVALID_NUMBER_OPS,
        ERROR_INVALID_NUMBER_ROUTER_SWAP_ARGS, ERROR_INVALID_NUMBER_SWAP_OPS, ERROR_MISSING_AMOUNT,
        ERROR_MISSING_AMOUNT_IN, ERROR_MISSING_FUNCTION_NAME, ERROR_MISSING_NUMBER_OPS,
        ERROR_MISSING_NUMBER_SWAP_OPS, ERROR_MISSING_PAIR_ADDR, ERROR_MISSING_TOKEN_ID,
        ERROR_ROUTER_SWAP_0_PAYMENTS, ERROR_SMART_SWAP_TWO_ARGUMENTS,
    },
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

    // Example of how the SmartSwaps arguments would be structured:
    // args = [
    //     "2",           // num_operations
    //     "20",          // percentage for first operation (20%)
    //     "2",           // num_swap_ops for first operation
    //     "pair_addr_1", "swapTokensFixedInput", "UTK", "200",  // first swap
    //     "pair_addr_2", "swapTokensFixedInput", "EGLD", "10",   // second swap
    //     "80",          // percentage for second operation (80%)
    //     "1",           // num_swap_ops for second operation
    //     "pair_addr_3", "swapTokensFixedInput", "EGLD", "800", // single swap
    // ]

    fn smart_swap(
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
            args.len() >= SMART_SWAP_MIN_ARGS_LEN,
            ERROR_SMART_SWAP_TWO_ARGUMENTS
        );

        let payment_in = payment_for_current_task.unwrap_esdt();
        let mut amount_out = BigUint::zero();
        let token_out = self.get_token_out_from_smart_swap_args(args.clone());

        let mut args_iter = args.into_iter();

        // First argument: number of swap operations
        let num_operations_buf = args_iter
            .next()
            .unwrap_or_else(|| sc_panic!(ERROR_MISSING_NUMBER_OPS));
        let num_operations = num_operations_buf
            .parse_as_u64()
            .unwrap_or_else(|| sc_panic!(ERROR_INVALID_NUMBER_OPS));

        let mut final_payments = ManagedVec::new();
        let mut acc_ammount_in = BigUint::zero();

        // Parse each operation
        for _ in 0..num_operations {
            // Parse: partial_amount_in, num_swap_ops, then swap operations
            let partial_amount_in = BigUint::from(
                args_iter
                    .next()
                    .unwrap_or_else(|| sc_panic!(ERROR_MISSING_AMOUNT_IN)),
            );

            acc_ammount_in += &partial_amount_in;

            let num_swap_ops_buf = args_iter
                .next()
                .unwrap_or_else(|| sc_panic!(ERROR_MISSING_NUMBER_SWAP_OPS));
            let num_swap_ops = num_swap_ops_buf
                .parse_as_u64()
                .unwrap_or_else(|| sc_panic!(ERROR_INVALID_NUMBER_SWAP_OPS));

            // Build swap arguments for this operation
            let mut operation_swap_args = ManagedVec::new();
            for _ in 0..num_swap_ops {
                // Each swap operation: pair_address, function_name, token_id, amount
                operation_swap_args.push(
                    args_iter
                        .next()
                        .unwrap_or_else(|| sc_panic!(ERROR_MISSING_PAIR_ADDR)),
                );
                operation_swap_args.push(
                    args_iter
                        .next()
                        .unwrap_or_else(|| sc_panic!(ERROR_MISSING_FUNCTION_NAME)),
                );
                operation_swap_args.push(
                    args_iter
                        .next()
                        .unwrap_or_else(|| sc_panic!(ERROR_MISSING_TOKEN_ID)),
                );
                operation_swap_args.push(
                    args_iter
                        .next()
                        .unwrap_or_else(|| sc_panic!(ERROR_MISSING_AMOUNT)),
                );
            }

            let operation_payment = EsdtTokenPayment::new(
                payment_in.token_identifier.clone(),
                payment_in.token_nonce,
                partial_amount_in,
            );

            let mut operation_result = self.multi_pair_swap(operation_payment, operation_swap_args);
            // Remove last payment; only residuals added
            let partial_payment_out = operation_result.take(operation_result.len() - 1);
            amount_out += partial_payment_out.amount;
            final_payments.append_vec(operation_result);
        }

        require!(
            acc_ammount_in <= payment_in.amount,
            ERROR_ACC_AMOUNT_EXCEEDS_PAYMENT_IN
        );

        // Handle remaining amount if total percentage < 100%
        if acc_ammount_in < payment_in.amount {
            let remaining_amount = payment_in.amount - acc_ammount_in;

            final_payments.push(EsdtTokenPayment::new(
                payment_in.token_identifier.clone(),
                payment_in.token_nonce,
                remaining_amount,
            ));
        }

        // Return the others to payments_to_return
        payments_to_return.append_vec(final_payments);

        EgldOrEsdtTokenPayment::new(token_out, 0, amount_out)
    }

    fn get_token_out_from_smart_swap_args(
        &self,
        args: ManagedVec<ManagedBuffer>,
    ) -> EgldOrEsdtTokenIdentifier<Self::Api> {
        let token_out_buffer = args
            .get(args.len() - ROUTER_TOKEN_OUT_FROM_END_OFFSET)
            .clone_value();
        EgldOrEsdtTokenIdentifier::esdt(token_out_buffer)
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
