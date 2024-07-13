use crate::{types::CallEncoder, OrderRequirements, LOG_TARGET};
use core::marker::PhantomData;
use frame_support::weights::WeightToFee;
use scale_info::prelude::vec;
use sp_runtime::{traits::Get, DispatchResult};
use xcm::latest::prelude::*;

/// Type able to dispatch coretime orders to the RegionX parachain.
pub trait OrderDispatcher {
	/// Constructs the order based on the requirements and dispatches it to the RegionX parachain.
	fn dispatch(requirements: OrderRequirements) -> DispatchResult;
}

pub struct DefaultOrderDispatcher<T: crate::Config + pallet_xcm::Config>(PhantomData<T>);
impl<T: crate::Config + pallet_xcm::Config> OrderDispatcher for DefaultOrderDispatcher<T> {
	fn dispatch(requirements: OrderRequirements) -> DispatchResult {
		let call = T::CallEncoder::order_creation_call(requirements);

		// `ref_time` = 53372000, we will round up to: 100000000.
		// `proof_size` = 6156, we will round up to: 10000.
		let call_weight = Weight::from_parts(100_000_000, 10_000);
		let fee = T::WeightToFee::weight_to_fee(&call_weight);

		let message = Xcm(vec![
			Instruction::WithdrawAsset(
				MultiAsset { id: Concrete(MultiLocation::parent()), fun: Fungible(fee.into()) }
					.into(),
			),
			Instruction::BuyExecution {
				fees: MultiAsset {
					id: Concrete(MultiLocation::parent()),
					fun: Fungible(fee.into()),
				},
				weight_limit: Unlimited,
			},
			Instruction::Transact {
				origin_kind: OriginKind::SovereignAccount,
				require_weight_at_most: call_weight,
				call: call.into(),
			},
		]);

		match pallet_xcm::Pallet::<T>::send_xcm(
			Here,
			<T as crate::Config>::RegionXLocation::get(),
			message,
		) {
			Ok(_) => log::info!(
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
