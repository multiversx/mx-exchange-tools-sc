# Composable Tasks SC

## Overview

This smart contract enables users to compose multiple actions while interacting with various Smart Contracts from MultiversX ecosystem, including xExchange.
It streamlines the process of interacting with WrapEGld and xExchange and provides a convenient way to perform multiple actions in a single transaction on the blockchain.

Complex actions are formed of multiple tasks. The tasks are performed synchronously, one after the other.
Example of tasks:
- wrapEGLD
- unwrapEGLD
- Swap
- Send EGLD/ESDT to third party


Example of actions:
- Wrap EGLD & send to third party
- Swap ESDT to wEGLD & unwrap to EGLD
- Wrap EGLD & swap to ESDT & send to third party

> **_Note:_** If the last task is **not** `Send tokens`, the resulted payment will be returned to the caller. Otherwise, the payment goes to the destination. 

## Task Structure

A task receives an `EgldOrEsdtPayment` and outputs one as well.
The resulted `EgldOrEsdtPayment` is forwarded to the next task.
If one task fails, the whole process will fail.

The `composeTasks` endpoint:
```
    #[payable("*")]
    #[endpoint(composeTasks)]
    fn compose_tasks(
        &self,
        opt_dest_addr: OptionalValue<ManagedAddress>,
        tasks: MultiValueEncoded<MultiValue2<TaskType, ManagedVec<ManagedBuffer>>>,
    )
```

where `TaskType`:

```
pub enum TaskType {
    WrapEGLD,
    UnwrapEGLD,
    Swap,
    SendEsdt,
}
```


> **_WARNING:_**  If you provide a wrong destination address, the payment will be sent there.

Most of the tasks don't require arguments, but some do (like `Swap`). An example of calling `Swap` task:

```
                let mut swap_args = ManagedVec::new();
                swap_args.push(managed_buffer!(TOKEN_ID));
                swap_args.push(managed_buffer!(b"1"));

                let mut tasks = MultiValueEncoded::new();
                tasks.push((TaskType::Swap, swap_args).into());
```