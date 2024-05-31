/// The coretime region requirements for the parachain.
///
/// This will describe the requirements in the coretime order.
#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct Requirements {
	/// The timeslice at which the Region begins.
	pub begin: Timeslice,
	/// The timeslice at which the Region ends.
	pub end: Timeslice,
	/// The minimum fraction of the core that the region should occupy.
	pub core_occupancy: PartsOf57600,
}
