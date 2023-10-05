# Composable Tasks SC

## Overview

This smart contract enables users to compose multiple actions while interacting with various Smart Contracts from MultiversX ecosystem, including xEchange.
It streamlines the process of interacting with WrapEGld and xExchange and provides a convenient way to perform multiple actions in a single transaction on the blockchain.

Complex actions are formed of multiple tasks. The tasks are performed synchronously, one after the other.
Example of tasks:
- wrapEGLD
- unwrapEGLD
- Swap
- Send ESDT to third party

```
pub enum TaskType {
    WrapEGLD,
    UnwrapEGLD,
    Swap,
    SendEsdt,
}
```

Example of actions:
- Wrap EGLD & send to third party
- Swap ESDT to wEGLD & unwrap to EGLD
- Wrap EGLD & swap to ESDT & send to third party

## Task Structure

A task receives an `EgldOrEsdtPayment` and outputs one as well.
The resulted `EgldOrEsdtPayment` is forwarded to the next task.
If one task fails, the whole process will fail.

## Compose Tasks
```
    #[payable("*")]
    #[endpoint(composeTasks)]
    fn compose_tasks(
        &self,
        opt_dest_addr: OptionalValue<ManagedAddress>,
        tasks: MultiValueEncoded<MultiValue2<TaskType, ManagedVec<ManagedBuffer>>>,
    )
```

> **_WARNING:_**  If you provide a wrong destination address, the payment will be sent there.
