use crate::external_sc_interactions;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TypeAbi, TopEncode, TopDecode, PartialEq, ManagedVecItem)]
pub enum TaskType {
    WrapEGLD,
    UnwrapEGLD,
    Swap,
    RouterSwap,
    SendEsdt,
}

// #[derive(TypeAbi, TopEncode, ManagedVecItem)]
// pub struct EndpointCall<M: ManagedTypeApi> {
//     pub endpoint_name: ManagedBuffer<M>,
//     pub endpoint_args: MultiValueEncoded<M, ManagedBuffer<M>>,
// }

pub type EndpointCall<M> = MultiValue3<ManagedAddress<M>, ManagedBuffer<M>, MultiValueEncoded<M, ManagedBuffer<M>>>;

#[multiversx_sc::module]
pub trait TaskCall:
    external_sc_interactions::pair_actions::PairActionsModule
    + external_sc_interactions::farm_actions::FarmActionsModule
    + external_sc_interactions::router_actions::RouterActionsModule
    + external_sc_interactions::wegld_swap::WegldSwapModule
{
    #[payable("*")]
    #[endpoint(composeTasks)]
    fn compose_tasks(
        &self,
        opt_dest_addr: OptionalValue<ManagedAddress>,
        tasks: MultiValueEncoded<MultiValue2<TaskType, ManagedVec<ManagedBuffer>>>,
    ) {
        let payment = self.call_value().egld_or_single_esdt();
        let mut payment_for_next_task = payment;

        let caller = self.blockchain().get_caller();

        #[allow(clippy::redundant_clone)] // clippy is dumb
        let dest_addr = match opt_dest_addr {
            OptionalValue::Some(opt_caller) => opt_caller,
            OptionalValue::None => caller.clone(),
        };

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
                TaskType::RouterSwap => {
                    require!(
                        !payment_for_current_task.token_identifier.is_egld(),
                        "EGLD can't be swapped!"
                    );
                    let payment_in = payment_for_current_task.unwrap_esdt();
                    self.multi_pair_swap(payment_in, args)
                }
                _ => {
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
        self.send().direct(
            &caller,
            &payment_for_next_task.token_identifier,
            payment_for_next_task.token_nonce,
            &payment_for_next_task.amount,
        );
    }

    #[payable("*")]
    #[endpoint(composeTasksv2)]
    fn compose_tasks_v2(
        &self,
        opt_final_dest_payment: OptionalValue<ManagedAddress>,
        tasks: MultiValueEncoded<EndpointCall<Self::Api>>,
    ) {
        let payment = self.call_value().egld_or_single_esdt();
        let payment_for_next_task = payment;

        let caller = self.blockchain().get_caller();

        #[allow(clippy::redundant_clone)] // clippy is dumb
        let final_payment_addr = match opt_final_dest_payment {
            OptionalValue::Some(opt_caller) => opt_caller,
            OptionalValue::None => caller.clone(),
        };

        for task in tasks.into_iter() {
            let (dest_addr, endpoint_name, args) = task.into_tuple();

            let payment_for_current_task = payment_for_next_task.clone();
            let _payment_for_next_task = self
                .send()
                .contract_call::<EsdtTokenPayment>(dest_addr, endpoint_name)
                .with_egld_or_single_esdt_transfer(payment_for_current_task.clone())
                .with_raw_arguments(args.to_arg_buffer())
                .transfer_execute();
        }

        self.send().direct(
            &final_payment_addr,
            &payment_for_next_task.token_identifier,
            payment_for_next_task.token_nonce,
            &payment_for_next_task.amount,
        );
    }
}
