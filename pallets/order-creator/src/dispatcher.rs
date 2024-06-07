// TODO: should this be a common file which is used by all the modules?
use crate::{types::OrderCallCreator, OrderRequirements, LOG_TARGET};
use codec::Encode;
use core::marker::PhantomData;
use frame_support::weights::WeightToFee;
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
		let call = T::OrderCallCreator::create_order_call(requirements);

		// `ref_time` = 53372000, we will round up to: 100000000.
		// `proof_size` = 6156, we will round up to: 7000.
		let call_weight = Weight::from_parts(100000000, 7000);
		let fee = T::RegionXWeightToFee::weight_to_fee(&call_weight);

		let message = Xcm(vec![
			Instruction::BuyExecution {
				fees: (MultiLocation::parent(), fee).into(),
				weight_limit: Unlimited, // TODO
			},
			Instruction::Transact {
				origin_kind: OriginKind::SovereignAccount,
				require_weight_at_most: call_weight,
				call: call.into(),
			},
		]);

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
