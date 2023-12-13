use core::convert::TryFrom;

use crate::external_sc_interactions;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TypeAbi, TopEncode, TopDecode, PartialEq, ManagedVecItem)]
pub enum TaskType {
    WrapEGLD,
    UnwrapEGLD,
    Swap,
    RouterSwap,
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
        expected_token_out: EgldOrEsdtTokenPayment,
        tasks: MultiValueEncoded<MultiValue2<TaskType, ManagedVec<ManagedBuffer>>>,
    ) {
        let mut payment_for_next_task = self.call_value().egld_or_single_esdt();

        let caller = self.blockchain().get_caller();

        for task in tasks.into_iter() {
            let (task_type, args) = task.into_tuple();

            let payment_for_current_task = payment_for_next_task.clone();

            payment_for_next_task = match task_type {
                TaskType::WrapEGLD => self.wrap_egld(payment_for_current_task),
                TaskType::UnwrapEGLD => self.unwrap_egld(payment_for_current_task),
                TaskType::Swap => {
                    require!(
                        !payment_for_current_task.token_identifier.is_egld(),
                        "EGLD can't be swapped!"
                    );
                    let payment_in = payment_for_current_task.unwrap_esdt();

                    require!(args.len() == 2, "Swap requires only 2 argumnets");

                    let token_out = TokenIdentifier::from(args.get(0).clone_value());
                    let min_amount_out = BigUint::from(args.get(1).clone_value());

                    self.perform_tokens_swap(
                        payment_in.token_identifier,
                        payment_in.amount,
                        token_out,
                        min_amount_out,
                    )
                    .into()
                }
                TaskType::RouterSwap => {
                    require!(
                        !payment_for_current_task.token_identifier.is_egld(),
                        "EGLD can't be swapped!"
                    );
                    let payment_in = payment_for_current_task.unwrap_esdt();
                    self.multi_pair_swap(payment_in, args)
                }
                TaskType::SendEgldOrEsdt => {
                    self.require_min_expected_token(&expected_token_out, &payment_for_current_task);

                    let dest_addr =
                        ManagedAddress::try_from(args.get(0).clone_value()).unwrap_or(caller);

                    self.send().direct(
                        &dest_addr,
                        &payment_for_current_task.token_identifier,
                        payment_for_current_task.token_nonce,
                        &payment_for_current_task.amount,
                    );
                    return;
                }
            };
        }
        self.require_min_expected_token(&expected_token_out, &payment_for_next_task);

        self.send().direct(
            &caller,
            &payment_for_next_task.token_identifier,
            payment_for_next_task.token_nonce,
            &payment_for_next_task.amount,
        );
    }

    fn require_min_expected_token(
        &self,
        expected_token: &EgldOrEsdtTokenPayment,
        token_out: &EgldOrEsdtTokenPayment,
    ) {
        require!(
            expected_token.token_identifier == token_out.token_identifier
                && expected_token.amount <= token_out.amount,
            "The output token is less than minimum required by user!"
        );
    }
}
