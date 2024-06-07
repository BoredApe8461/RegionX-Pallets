// TODO: should this be a common file which is used by all the modules?
use crate::{OrderRequirements, LOG_TARGET};
use core::marker::PhantomData;
use sp_runtime::DispatchResult;
use xcm::latest::prelude::*;

/// Type able to dispatch coretime orders to the RegionX parachain.
pub trait OrderDispatcher {
	/// Constructs the order based on the requirements and dispatches it to the RegionX parachain.
	fn dispatch(requirements: OrderRequirements) -> DispatchResult;
}

pub struct DefaultOrderDispatcher<T: crate::Config + pallet_xcm::Config>(PhantomData<T>);
impl<T: crate::Config + pallet_xcm::Config> OrderDispatcher for DefaultOrderDispatcher<T> {
	fn dispatch(requirements: OrderRequirements) -> DispatchResult {
		let message = Xcm(vec![]);

		match pallet_xcm::Pallet::<T>::send_xcm(Here, MultiLocation::parent(), message) {
				Ok(_) => log::debug!(
				target: LOG_TARGET,
				"Coretime order sent successfully"
			),
			Err(e) => log::error!(
				target: LOG_TARGET,
				"Failed to send coretime order: {:?}",
				e
			),
		}
		Ok(())
	}
}
