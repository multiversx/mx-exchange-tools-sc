# sc-auto-farm

xExchange introduced the concept of energy, locked tokens, boosted farms, boosted staking contracts (little bit later), fees collector. In a few ways we have simplified the interactions the user has to do: he does not need to commit x% of the locked tokens for a specific farm to earn boosted rewards ( in case of yFi, Curve, TraderJoe, the user has to choose in which contract does he put his locked tokens to earn boosted rewards). In our case users will benefit from boosted rewards for their complete energy position for all the farms, staking contracts and fees collectors they have entered.

However, by implementing this and in order for the user to benefit maximally from the rewards, he would need to interact with the contracts every week - especially on Friday they should claim all their rewards and update the energy on all the contracts. This is not so hard for a heavy DeFi user, but for normal users, this is not easy. (especially that they use a mobile browser to interact with the xExchange). 

Furthermore, if a user does not claim his rewards every 4 weeks at least, he will lose some of those. If the user makes some wrong transactions, he might lose a part of his boosted rewards (there are multiple things here). There are several security issues why we choose an architecture like this at the base contracts, and we wish for those to stay like that.  Right now, contracts cannot hold energy at all (protected at energy-factory contract), we choose this to not concentrate all energy in a few contracts, which might use strategies which are not beneficial for the ecosystem.

In order to enhance the user experience for non DeFi users, we propose a set of SCs and a paid service through which users can easily enjoy all the benefits, maximized rewards, compounding, without the need of interaction every week.

##Let’s start with the basics feature we need:
1. Claim rewards every week
2. Update energy every week
3. Compound on farms (use earned rewards from fees collector to enter farms)
4. Compound on staking and metastaking contracts

##The basic idea: 
Create a smart contract in which the user deposits his farm, metastaking positions. The contract is whitelisted in the xExchange contracts in order to act as a proxy for the user ( he calls every endpoint with optional argument user_address, so the base contracts will read the user’s energy every time). Every week or every day an external service iterates over all the users in the contract calling claim rewards, update energy, compound and everything needed as strategy. The smart contract should be general enough to allow different kinds of strategies - but he is never allowed to take the deposited tokens of the users.
As the external service needs tokens in order to pay for the gas and electricity, we put a 1% FEE of all the rewards from the users to remain in the contract. The owner of the contract will be able to claim it at any point. 

##The user flow - this has to be the simplest:
1. User deposits farm tokens or metastaking tokens - it can be a multi transfer or multiple transactions.
   a. The contract registers all these tokens under the key - userAddress
2. We should make it easier - one click creation of farm tokens
   a. User deposits X token and says he wants to enter in Farm/MetaStaking of Y-Z tokens (Y can be the same as X). Users can come with wrappedEGLD and say they want to enter the metastaking of EGLD-RIDE. Or can come with USDC and say he wants to enter the farm contract of EGLD-XMEX with maximum energy.
   b. Contract will interact with the Liquidity Pool contract to buy with 50% of X tokens the necessary Y tokens and 50% of X tokens to buy the necessary Z tokens. After that it will enter into the specified FARM contract and can enter into the metaStaking contract as well.
   c. The contract will add the resulting farm/metastaking position to the storage held for the user, under the user address.
3. When a user deposits he agrees that a part of his rewards will go be a FEE to the service provider. 
4. There is nothing else to be done by the user, he just goes on with his life and claims all rewards or exits some/all positions after a few months.
5. Users who do not have farm tokens only XMEX should be able to participate in, they have to sign a single transaction and enter the contract. In that case the contract needs to be claimed from fees collector and metabonding only.


##Let’s go to more details about the contract:

###depositFarmTokens:
User call this with farm/proxy-dex-farm/metastaking/staking positions. The contract adds the given position to the user storage.

###makeMeAPosition:
From a single token: one ESDT transfer and the argument is the pair he wants to enter. Contract does 50% buy tokenA, 50% buy tokenB (if tokenA == ESDT transfer - no need to buy tokenA), Enter liquidity with results, Enter Farm with LP, Enter Metastaking if we have.

####claimRewardsForUsers - can be called by whitelisted addresses only (the service will call this):
Will iterate over all the users and will iterate over all the positions of each user and claim rewards for it. Plus it needs to claim rewards from metabonding contract and claim rewards from fees collector.
The contract calls each farm/staking/proxy-dex/metastaking with user_position and user_address as additional argument, always - as we claim in the name of the user all rewards - to get the boosted rewards as well. Because of this the autofarm contract, has to be whitelisted in each of the aforementioned contracts.

We need a MAP of TokenID - Contract Address To claim Rewards
When the process of iterating over all positions from one user, it will call updateEnergy for all the contracts the user is in.

All claimed rewards will remain in user storage.

###claimRewards: 
User claims rewards from the autofarm contract. Contract sends all accumulated rewards to the user.

###exitAutoFarm:
User calls it and says what position and how much of that position he wants to exit. The user receives those positions. After that, exiting liquidity, exiting farms and all those things remains to be done by the user on the xExchange main page.

###fullExitPosition:
User calls it and says which position and how much of that he wants to fully exit. The contract will exit metastaking/proxy-dex/farm/staking, exit liquidity and send all gathered tokens from that exit to the user.
In case of metaStaking - the user will have an unstaked position in the staking contract, the user will have to use the main page of xExchange to unbond from there after 10 days.

##Details about the service:
The end of the week in terms of boosted rewards happens always on Friday at epoch change. We should run the service Friday morning when there are not many transactions.
###claimRewardsForUsers do not need any params - as all the logic is in the contract.

