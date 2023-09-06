# Energy DAO Smart Contract

## Abstract

With the latest update to the DEX contracts, smart contracts can now obtain Energy and leverage this concept. The __Energy DAO SC__ template is an example and a starting point on how projects can provide utility for users by using the concept of Energy.

## Introduction and important notes

The template is an independent SC and integrates multiple DEX contracts. Users can deposit their tokens in the contract for staking purposes, while the contract receives and locks MEX tokens to gather energy that benefits all users. The SC always keeps one aggregated position for each feature and computes rewards using a rewards-per-share algorithm. Each user position is represented by specific tokens issued by the __Energy DAO SC__. The config module contains all the configs and general utilities of the SC. The contract keeps the rewards from the __Fees Collector__ contract as rewards for providing energy. Furthermore, it imposes a fee on every user exit action.

While being in a way a variation of the __auto-farm SC__, it was designed as a completely independent contract in the mx-exchange-tools repo. It can be cloned directly, without the need to import any other contract. The only external dependency is the xExchange suite of contracts, that are referenced through a Github commit hash from the latest version of that repo.

A very important aspect here is that, with the current protocol design, in order to work as intended, the __Energy DAO SC__ must be deployed on the same shard as the DEX, in order to use intrashard contract calls and have syncronous, realtime SC results from the xExchange contracts.
Later on, with the launch of the AsyncV2 functionality, these kinds of contracts will be able to be deployed in other shards as well, as the protocol will support multiple asyncCalls.

Another point that needs mentioning, is that the current DEX SCs design presents a challenge for an __Energy DAO SC__, which comprises both __Farm__ and __Metastaking__ implementations for the same underlying tokens. Combining these two approaches results in a loss of rewards for one of the farms, as boosted rewards are given per account, and cannot be claimed with one aggregated position consisting of both farm and metastaking tokens. To address this issue, the __Energy DAO SC__ template does not allow to use both options at the same time. In other words, you cannot enter a __Farm__ contract, if for that __Farm__ you have defined the __Metastaking__ contract as well. Of course, each project can define its own custom logic regarding the rewards distribution and may allow both implementations to work simultaneously.

## Key implementation aspects

- The __Energy DAO__ integrates multiple DEX contracts, including __Farms__, __Metastaking__ (Farm Staking), __Fees Collector__, __Energy Factory__, as well as other smaller utility contracts.
- The SC always keeps one aggregated position for each feature (__Farms__ and __Metastaking__), and computes rewards using a rewards-per-share algorithm.
- Each user position is represented by specific tokens issued by the Energy DAO SC. There are tokens for both Farm & Metastaking current positions, as well as tokens for unbonding positions.
- The tokens are storing different metadata according to the user position, including the position's __rps__.
- The __Farms__ integration covers all 3 interaction points of the farm contract, including __enter_farm__, __exit_farm__ (with a 7 days unbonding period) and __claim_rewards__ as well, which aggregates all rewards and distributes them using an internal __rps__ computation.
- The __Metastaking__ integration resembles pretty much with the __Farms__ integration, with a few differences, including a double __rps__ computation, for each reward token, as well as a different unbonding implementation, in line with the __Metastaking__ SC logic.
- This SC template keeps the rewards from the __Fees Collector__ contract as rewards for providind Energy. Also, while entering the SC and claiming rewards are penalty free, a fee of __x%__ is imposed on every user exit action (the fee percentage is subject to change for each project individually).

During the entire SC implementation, every time a DEX contract is called and the respective endpoins require the opt_original_caller argument, the value __OptionalValue::None__ is passed, as we want all the benefits of the integration to be sent to __Energy DAO__ contract. Later, the contract can manage how these rewards are computed and further distributed.

## Farm integration

The __Energy DAO__ __Farm__ integration refers to the following workflow: User A provides a farming position (LP token) and the DAO SC enters the DEX farm contract. Then a second user B does the same thing, at which moment the DAO contract enters with both the current position and user B's position, always maintaining an aggregated farm position. The users positions are kept using a new __WrappedFarmToken__, issued by the __Energy DAO SC__. As new rewards are accumulated, they are stored in the contract and a __reward_per_share__ computation is saved as the rewards pool increases. The __WrappedFarmToken__ contains data about the __rps__ computed at the moment when the user entered the SC, and with that token, the user can claim his corresponding rewards. Because the rewards are given in XMEX, they are always merged as they are accumulated, and when they are sent to the users, they are first wrapped, in order to be transferable (Wrapped XMEX can only be unwrapped by user accounts). In the end, the user can choose to exit the __Energy DAO SC__, and after an unbonding period that must pass, a fee is applied on the farming position, before the user receives his tokens.

This template contract splits the __Farm__ integration in 2 different files, for better readability. One with the actual user interactions (the endpoints), where all the custom computation are done, and another one with the more generic actions regarding the DEX farm contract integration and any other general functions needed on this part.

## Metastaking integration

The __Metastaking__ integration is quite similar to the __Farms__ integration. That being said, there are still a few different nuances, especially regarding rewards computation (there are now 2 separate reward tokens and for that we have 2 separate rps amounts, one for each token) and a different unstake & unbond mechanism (as this differs from the farm logic, by being imposed by the DEX contract).

## Locked token integration

This __Energy DAO__ contract template was designed with the following workflow regarding the accumulation of Energy
- The owner buys MEX tokens and sends them to the __Energy DAO SC__
- The contract then locks & energizes the account
- For providing the tokens that are now locked, the owner is entitled to rewards from the __Fees Collector__, as well as the exit fees (the percentage can be changed by each project) from users that use the __Energy DAO__ contract.
- The owner can further extend the locking period of the tokens, in order to maximize the SC's Energy.

## Testing

The __Energy DAO SC__ was tested through various unit tests, that were conducted on top of a complete setup of the xExchange suite of contracts. Specifically, all the involved DEX contracts (like pair, farm, farm-staking, farm-staking-proxy, energy factory & so on) were set up from scratch, so the testing scenario could follow a complete flow where the owner locks his tokens through the SC in order to get Energy for the contract, and users provide liquidity in the pair contract, to later enter farm or metastaking, claim rewards and exit the __Energy DAO__ contract.

Tip. In order to be able to have a complete step-by-step debugging layout, all the Github references from the main `Cargo.toml` file need to be updated to a local DEX repo path, as shown below.
```rust
[dependencies.pair]
path = "../../mx-exchange-sc/dex/pair"
```

## Conclusion

The __Energy DAO SC__ template is a powerful tool for developers looking to create smart contracts that leverage the Energy concept to benefit users. By exploring the __Energy DAO__ contract's structure and configuration, developers can further customize the template to meet their specific needs.
