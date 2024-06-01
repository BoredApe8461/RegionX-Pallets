// TODO: should this be a common file which is used by all the modules?
use crate::OrderRequirements;

/// Type able to dispatch coretime orders to the RegionX parachain.
pub trait OrderDispatcher {
	/// Constructs the order based on the requirements and dispatches it to the RegionX parachain.
	fn dispatch(requirements: OrderRequirements);
}
