use crate::external_sc_interactions;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TypeAbi, TopEncode, TopDecode, PartialEq, ManagedVecItem)]
pub enum TaskType {
    WrapEGLD,
    UnwrapEGLD,
    Swap,
    SendEsdt,
}

#[multiversx_sc::module]
pub trait TaskCall:
    external_sc_interactions::pair_actions::PairActionsModule
    + external_sc_interactions::farm_actions::FarmActionsModule
    + external_sc_interactions::wegld_swap::WegldSwapModule
{
    #[payable("*")]
    #[endpoint(composeTasks)]
    fn compose_tasks(
        &self,
        final_destination: ManagedAddress,
        tasks: MultiValueEncoded<MultiValue2<TaskType, ManagedVec<ManagedBuffer>>>,
    ) {
        let payment = self.call_value().egld_or_single_esdt();
        let mut payment_for_next_task = payment;

        for task in tasks.into_iter() {
            let (task_type, args) = task.into_tuple();

            let payment_for_current_task = payment_for_next_task.clone();

            payment_for_next_task = match task_type {
                TaskType::WrapEGLD => self.wrap_egld(),
                TaskType::UnwrapEGLD => self.unwrap_egld(),
                TaskType::Swap => {
                    require!(
                        !payment_for_current_task.token_identifier.is_egld(),
                        "EGLD can't be swapped!"
                    );
                    let payment_in = payment_for_current_task.unwrap_esdt();

                    let token_out = TokenIdentifier::from(args.get(0).clone_value());
                    let min_amount_out = BigUint::from(args.get(1).clone_value());

                    self.perform_tokens_swap(
                        payment_in.token_identifier.clone(),
                        payment_in.amount,
                        token_out,
                        min_amount_out,
                    )
                    .into()
                }
                TaskType::SendEsdt => {
                    self.send().direct(
                        &final_destination,
                        &payment_for_current_task.token_identifier,
                        payment_for_current_task.token_nonce,
                        &payment_for_current_task.amount,
                    );
                    return;
                }
            };
        }
    }
}