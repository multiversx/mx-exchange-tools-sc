use pair::config::MAX_PERCENTAGE;

use crate::{
    compose_tasks::PaymentsVec,
    config,
    errors::{
        ERROR_ACC_AMOUNT_EXCEEDS_PAYMENT_IN, ERROR_CANNOT_SWAP_EGLD, ERROR_INCORRECT_ARGS,
        ERROR_INVALID_NUMBER_SWAP_OPS, ERROR_INVALID_TOKEN_ID, ERROR_MISSING_AMOUNT,
        ERROR_MISSING_AMOUNT_IN, ERROR_MISSING_FUNCTION_NAME, ERROR_MISSING_NUMBER_SWAP_OPS,
        ERROR_MISSING_PAIR_ADDR, ERROR_MISSING_TOKEN_ID, ERROR_SMART_SWAP_ARGUMENTS,
        ERROR_SMART_SWAP_TOO_MANY_OPERATIONS, ERROR_WRONG_RETURNED_TOKEN_IDENTIFIER,
        ERROR_ZERO_AMOUNT,
    },
    events, external_sc_interactions,
};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub const SMART_SWAP_ARGS_LEN: usize = 5;
pub const SMART_SWAP_MIN_ARGS_LEN: usize = 7;
pub const SMART_SWAP_MAX_OPERATIONS: u64 = 10;
pub const ROUTER_TOKEN_OUT_FROM_END_OFFSET: usize = 2;
pub const MAX_SWAPS_PER_OPERATION: u64 = 10;
pub const NUM_OPERATIONS_ARG: usize = 1;
pub const MIN_SMART_SWAP_ARGS: usize = 4;
pub const FIXED_SMART_SWAP_ARGS_PER_OPERATION: usize = 2; // amount_in + num_swap_ops

#[type_abi]
#[derive(TopEncode)]
pub struct SmartSwapEvent<M: ManagedTypeApi> {
    caller: ManagedAddress<M>,
    token_in: TokenIdentifier<M>,
    amount_in: BigUint<M>,
    token_out: TokenIdentifier<M>,
    amount_out: BigUint<M>,
    fee_amount: BigUint<M>,
    block: u64,
    epoch: u64,
    timestamp: u64,
}

#[type_abi]
#[derive(TopEncode)]
pub struct SmartSwapInput<M: ManagedTypeApi> {
    payment_in: EsdtTokenPayment<M>,
    token_out: EgldOrEsdtTokenIdentifier<M>,
    num_operations: u64,
}

pub struct SmartSwapProcessOperation<M: ManagedTypeApi> {
    acc_amount_in: BigUint<M>,
    amount_out: BigUint<M>,
}

pub struct SmartSwapFee<M: ManagedTypeApi> {
    fee_taken: BigUint<M>,
    remaining_amount_after_fee: BigUint<M>,
}

pub struct SmartSwapResultInput<M: ManagedTypeApi> {
    caller: ManagedAddress<M>,
    payment_in: EsdtTokenPayment<M>,
    acc_amount_in: BigUint<M>,
    token_out: EgldOrEsdtTokenIdentifier<M>,
    remaining_amount_after_fee: BigUint<M>,
    fee_taken: BigUint<M>,
}

#[multiversx_sc::module]
pub trait SmartSwapModule:
    config::ConfigModule
    + events::EventsModule
    + external_sc_interactions::pair_actions::PairActionsModule
    + external_sc_interactions::router_actions::RouterActionsModule
    + external_sc_interactions::wegld_swap::WegldWrapModule
{
    // Example of how the SmartSwaps arguments would be structured:
    // args = [
    //     "2",           // num_operations
    //     "2_000",          // amount_in for first operation
    //     "2",           // num_swap_ops for first operation
    //     "pair_addr_1", "swapTokensFixedInput", "UTK", "200",  // first swap
    //     "pair_addr_2", "swapTokensFixedInput", "EGLD", "10",   // second swap
    //     "8_000",          // amount_in for second operation
    //     "1",           // num_swap_ops for second operation
    //     "pair_addr_3", "swapTokensFixedInput", "EGLD", "800", // single swap
    // ]

    fn smart_swap(
        &self,
        payment_for_current_task: EgldOrEsdtTokenPayment,
        payments_to_return: &mut PaymentsVec<Self::Api>,
        args: ManagedVec<ManagedBuffer>,
    ) -> EgldOrEsdtTokenPayment {
        let smart_swap_input =
            self.validate_and_parse_smart_swap_input(payment_for_current_task, &args);

        let caller = self.blockchain().get_caller();
        let mut args_iter = args.into_iter();
        let _ = args_iter.next(); // Skip the num_operations argument

        let smart_swap_process_operation = self.process_smart_swap_operations(
            &smart_swap_input.payment_in,
            &smart_swap_input.token_out.clone().unwrap_esdt(),
            smart_swap_input.num_operations,
            &mut args_iter,
            payments_to_return,
        );

        require!(
            smart_swap_process_operation.acc_amount_in <= smart_swap_input.payment_in.amount,
            ERROR_ACC_AMOUNT_EXCEEDS_PAYMENT_IN
        );

        self.handle_remaining_amount(
            &smart_swap_input.payment_in,
            &smart_swap_process_operation.acc_amount_in,
            payments_to_return,
        );

        let smart_swap_fee = self.calculate_and_apply_smart_swap_fee(
            &smart_swap_process_operation.amount_out,
            &smart_swap_input.token_out,
        );

        let smart_swap_result_input = SmartSwapResultInput {
            caller,
            payment_in: smart_swap_input.payment_in,
            acc_amount_in: smart_swap_process_operation.acc_amount_in,
            token_out: smart_swap_input.token_out,
            remaining_amount_after_fee: smart_swap_fee.remaining_amount_after_fee,
            fee_taken: smart_swap_fee.fee_taken,
        };

        self.finalize_smart_swap_result(smart_swap_result_input)
    }

    fn compose_smart_swap_operation_swap_args(
        &self,
        args_iter: &mut ManagedVecOwnedIterator<ManagedBuffer<Self::Api>>,
    ) -> ManagedVec<ManagedBuffer<Self::Api>> {
        let num_swap_ops_buf = args_iter
            .next()
            .unwrap_or_else(|| sc_panic!(ERROR_MISSING_NUMBER_SWAP_OPS));
        let num_swap_ops = num_swap_ops_buf
            .parse_as_u64()
            .unwrap_or_else(|| sc_panic!(ERROR_INVALID_NUMBER_SWAP_OPS));

        require!(
            num_swap_ops > 0 && num_swap_ops <= MAX_SWAPS_PER_OPERATION,
            ERROR_INVALID_NUMBER_SWAP_OPS
        );

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
        operation_swap_args
    }

    fn validate_and_parse_smart_swap_input(
        &self,
        payment_for_current_task: EgldOrEsdtTokenPayment,
        args: &ManagedVec<ManagedBuffer>,
    ) -> SmartSwapInput<Self::Api> {
        require!(
            !payment_for_current_task.token_identifier.is_egld(),
            ERROR_CANNOT_SWAP_EGLD
        );
        require!(
            args.len() >= SMART_SWAP_MIN_ARGS_LEN,
            ERROR_SMART_SWAP_ARGUMENTS
        );

        let payment_in = payment_for_current_task.unwrap_esdt();
        let token_out = self.get_token_out_from_smart_swap_args(args.clone());

        let num_operations_buf = args.get(0).clone_value();
        let num_operations = num_operations_buf
            .parse_as_u64()
            .unwrap_or_else(|| sc_panic!(ERROR_INVALID_NUMBER_SWAP_OPS));

        require!(
            num_operations > 0 && num_operations <= SMART_SWAP_MAX_OPERATIONS,
            ERROR_SMART_SWAP_TOO_MANY_OPERATIONS
        );

        // Validate total argument length based on structure
        self.validate_smart_swap_args_length(args, num_operations);

        SmartSwapInput {
            payment_in,
            token_out,
            num_operations,
        }
    }

    fn process_smart_swap_operations(
        &self,
        payment_in: &EsdtTokenPayment<Self::Api>,
        expected_token_out: &TokenIdentifier<Self::Api>,
        num_operations: u64,
        args_iter: &mut ManagedVecOwnedIterator<ManagedBuffer<Self::Api>>,
        payments_to_return: &mut PaymentsVec<Self::Api>,
    ) -> SmartSwapProcessOperation<Self::Api> {
        let mut acc_amount_in = BigUint::zero();
        let mut amount_out = BigUint::zero();

        for _ in 0..num_operations {
            let partial_amount_in = BigUint::from(
                args_iter
                    .next()
                    .unwrap_or_else(|| sc_panic!(ERROR_MISSING_AMOUNT_IN)),
            );

            require!(partial_amount_in > 0, ERROR_ZERO_AMOUNT);

            acc_amount_in += &partial_amount_in;

            let operation_swap_args = self.compose_smart_swap_operation_swap_args(args_iter);

            let operation_payment = EsdtTokenPayment::new(
                payment_in.token_identifier.clone(),
                payment_in.token_nonce,
                partial_amount_in,
            );

            let mut operation_result = self.multi_pair_swap(operation_payment, operation_swap_args);
            let partial_payment_out = operation_result.take(operation_result.len() - 1);

            require!(
                &partial_payment_out.token_identifier == expected_token_out,
                ERROR_WRONG_RETURNED_TOKEN_IDENTIFIER
            );

            amount_out += partial_payment_out.amount;
            payments_to_return.append_vec(operation_result);
        }

        SmartSwapProcessOperation {
            acc_amount_in,
            amount_out,
        }
    }

    fn handle_remaining_amount(
        &self,
        payment_in: &EsdtTokenPayment<Self::Api>,
        acc_amount_in: &BigUint<Self::Api>,
        payments_to_return: &mut PaymentsVec<Self::Api>,
    ) {
        if acc_amount_in >= &payment_in.amount {
            return;
        }

        let remaining_amount = &payment_in.amount - acc_amount_in;

        payments_to_return.push(EsdtTokenPayment::new(
            payment_in.token_identifier.clone(),
            payment_in.token_nonce,
            remaining_amount,
        ));
    }

    fn calculate_and_apply_smart_swap_fee(
        &self,
        amount_out: &BigUint<Self::Api>,
        token_out: &EgldOrEsdtTokenIdentifier<Self::Api>,
    ) -> SmartSwapFee<Self::Api> {
        let fee_percentage = self.smart_swap_fee_percentage().get();
        let fee_taken = amount_out * fee_percentage / MAX_PERCENTAGE;

        // Safely extract ESDT token identifier with proper validation
        require!(!token_out.is_egld(), ERROR_INVALID_TOKEN_ID);
        let token_esdt = token_out.clone().unwrap_esdt();

        self.smart_swap_fees(&token_esdt)
            .update(|total_fees| *total_fees += &fee_taken);

        let remaining_amount_after_fee = amount_out - &fee_taken;

        SmartSwapFee {
            fee_taken,
            remaining_amount_after_fee,
        }
    }

    fn finalize_smart_swap_result(
        &self,
        smart_swap_result_input: SmartSwapResultInput<Self::Api>,
    ) -> EgldOrEsdtTokenPayment<Self::Api> {
        let acc_payment_in = EsdtTokenPayment::new(
            smart_swap_result_input.payment_in.token_identifier,
            smart_swap_result_input.payment_in.token_nonce,
            smart_swap_result_input.acc_amount_in,
        );
        let payment_out = EgldOrEsdtTokenPayment::new(
            smart_swap_result_input.token_out,
            0,
            smart_swap_result_input.remaining_amount_after_fee,
        );

        self.emit_smart_swap_event(
            smart_swap_result_input.caller,
            acc_payment_in,
            payment_out.clone().unwrap_esdt(),
            smart_swap_result_input.fee_taken,
        );

        payment_out
    }

    fn get_token_out_from_smart_swap_args(
        &self,
        args: ManagedVec<ManagedBuffer>,
    ) -> EgldOrEsdtTokenIdentifier<Self::Api> {
        let args_len = args.len();
        require!(
            args_len > ROUTER_TOKEN_OUT_FROM_END_OFFSET,
            ERROR_INCORRECT_ARGS
        );
        let token_out_buffer = args
            .get(args_len - ROUTER_TOKEN_OUT_FROM_END_OFFSET)
            .clone_value();

        let token_out = EgldOrEsdtTokenIdentifier::esdt(token_out_buffer);
        require!(token_out.is_valid(), ERROR_INVALID_TOKEN_ID);

        token_out
    }

    // This is a simplified validation - the full validation happens during execution
    fn validate_smart_swap_args_length(
        &self,
        args: &ManagedVec<ManagedBuffer>,
        num_operations: u64,
    ) {
        let min_expected = NUM_OPERATIONS_ARG
            + num_operations as usize * (FIXED_SMART_SWAP_ARGS_PER_OPERATION + MIN_SMART_SWAP_ARGS);

        require!(args.len() >= min_expected, ERROR_SMART_SWAP_ARGUMENTS);
    }
}
