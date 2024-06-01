use codec::{Decode, Encode, MaxEncodedLen};
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_broker::{ConfigRecord, PartsOf57600, Timeslice};
use scale_info::TypeInfo;
use sp_runtime::traits::BlockNumberProvider;

/// Relay chain block number.
pub type RCBlockNumberOf<T> =
	<<T as crate::Config>::RCBlockNumberProvider as BlockNumberProvider>::BlockNumber;

pub type ConfigRecordOf<T> = ConfigRecord<BlockNumberFor<T>, RCBlockNumberOf<T>>;

/// Specifies the requirements of a Coretime order.
#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct OrderRequirements {
	/// The timeslice at which the Region begins.
	pub begin: Timeslice,
	/// The timeslice at which the Region ends.
	pub end: Timeslice,
	/// The minimum fraction of the core that the region should occupy.
	pub core_occupancy: PartsOf57600,
}

/// Generic Coretime region requirements for the parachain.
///
/// Based on this we will construct `OrderRequirements` per order.
///
/// Currently, we only support requirements based on core occupancy, not on region duration.
#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct GenericRequirements {
	/// The minimum fraction of the core that the region should occupy.
	pub core_occupancy: PartsOf57600,
}
