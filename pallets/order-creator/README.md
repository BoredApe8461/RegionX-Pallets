## Order Creator Pallet

### Context

There are currently two ways to interact with the RegionX parachain. One of them is through `pallet-market` and the other is through `orders-pallet`.

Most parachains strive to be decentralized autonomous entities that do not rely on any specific set of people. For this to be achieved, it is important for them to procure coretime in a decentralized and autonomous manner. 

One option is to procure coretime from the bulk market and continuously renew it. However, most parachains do not need an entire core allocated to them.  This is why coretime marketplaces like RegionX exist. They allow parachains to procure only the coretime they actually need, significantly lowering their operational costs.

As already mentioned, given that parachains are decentralized, transparent, and autonomous systems, interaction with a marketplace is inconvenient for them. Their actions on the market are transparent, making it easy for someone to front-run their actions, potentially harming the operation of the chain.

To solve this problem, we created the orders pallet, which provides a way for parachains to interact with the secondary market in a decentralized, community-driven way.

The way orders work is that every parachain periodically, at the start of a bulk period, posts its coretime requirements. This will indicate to coretime traders the demand that exists. Each order is initially allocated zero tokens. Anyone, including the parachain itself, can participate in order crowdfunding. The tokens collected from the participants will incentivize someone to fulfill the order, i.e., sell coretime to it. The crowdfunded amount will be allocated to the trader that fulfills the order.

There can be multiple incentives to fund the parachain's order. If people rely on the parachain's services, they might not need any additional incentive other than the risk associated with the parachain stopping execution. 

However, similar to how parachains used to reward the crowdloan participants in the legacy slot auction model, we can expect that parachains will reward contributors in a similar manner. For example, a parachain might offer a set amount of tokens from its treasury to reward participants.

We can also expect the parachain team to be consistent in order contribution since it is crucial for them that the parachain continues execution.

### Order creator pallet

The order creator pallet facilitates integration with the RegionX coretime marketplace.

It exposes several extrinsics through which the configured `T::AdminOrigin` can configure the order requirements, schedule the next order, and set the Coretime chain-related configuration.

To initialize the pallet, the `T::AdminOrigin` has to set all the configurations. This means they have to set the Coretime chain configuration, the timeslice at which to make the first order, as well as the coretime requirements of the parachain.

After the initial configuration the pallet will continously make coretime orders to the RegionX parachain at the start of every bulk period.
