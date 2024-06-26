//! Order creator pallet.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use pallet_broker::Timeslice;
use sp_runtime::SaturatedConversion;
use frame_support::pallet_prelude::Weight;

mod types;
pub use crate::types::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub trait WeightInfo {
	fn set_configuration() -> Weight;
	fn schedule_next_order() -> Weight;
	fn set_coretime_requirements() -> Weight;
}

mod dispatcher;
pub use crate::dispatcher::*;

const LOG_TARGET: &str = "runtime::order-creator";

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		pallet_prelude::*,
		traits::{
			fungible::{Inspect, Mutate},
			tokens::Balance,
			Get,
		},
		weights::WeightToFee,
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::BlockNumberProvider;
	use xcm::opaque::lts::MultiLocation;

	/// The module configuration trait.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The relay chain currency used for coretime procurement.
		type RelaychainCurrency: Mutate<Self::AccountId>;

		/// Relay chain balance type
		type RelaychainBalance: Balance
			+ Into<<Self::RelaychainCurrency as Inspect<Self::AccountId>>::Balance>
			+ Into<u128>;

		/// A means of getting the current relay chain block.
		///
		/// This is used for determining the current timeslice.
		type RCBlockNumberProvider: BlockNumberProvider;

		/// The RegionX parachain location to which the orders are sent.
		type RegionXLocation: Get<MultiLocation>;

		/// The admin origin for managing the order creation.
		type AdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// Type responsible for dispatching coretime orders to the RegionX parachain.
		type OrderDispatcher: OrderDispatcher;

		/// Type which will return the scale encoded call for creating an order.
		type CallEncoder: CallEncoder;

		/// Type for weight to fee conversion on the ReigonX parachain.
		type WeightToFee: WeightToFee<Balance = Self::RelaychainBalance>;

		/// Number of Relay-chain blocks per timeslice.
		#[pallet::constant]
		type TimeslicePeriod: Get<RCBlockNumberOf<Self>>;

		/// Weight Info
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// The current configuration of the Coretime chain.
	///
	/// Can be modified by the `AdminOrigin`.
	///
	/// *WARNING*: If the sale duration changes on the Coretime chain, the `AdminOrigin` should
	/// update the `Configuration` to match the new config on the Coretime chain. If not updated, we
	/// run the risk of creating redundant orders or missing an order creation.
	///
	/// Ideally, we would read the current sale state directly from the Coretime chain. However,
	/// that would require using something like ISMP and a relay infrastructure.
	#[pallet::storage]
	#[pallet::getter(fn configuration)]
	pub type Configuration<T: Config> = StorageValue<_, ConfigRecordOf<T>, OptionQuery>;

	/// The timeslice at which the next order should be made.
	///
	/// When setting up the pallet, if the parachain has already procured Coretime for the upcoming
	/// bulk period, this should be set to the start of the upcoming bulk period. Otherwise, we can
	/// set this to the start of the current bulk period so that we attempt to procure Coretime
	/// for the upcoming bulk period.
	///
	/// After initially set, the pallet will keep this up to date by itself.
	#[pallet::storage]
	#[pallet::getter(fn next_order)]
	pub type NextOrder<T: Config> = StorageValue<_, Timeslice, OptionQuery>;

	/// The coretime requirements for the parachain.
	///
	/// Orders will be made based on this.
	#[pallet::storage]
	#[pallet::getter(fn coretime_requirements)]
	pub type CoretimeRequirements<T: Config> = StorageValue<_, GenericRequirements, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Configuration of the coretime chain was set.
		ConfigurationSet { configuration: ConfigRecordOf<T> },
		/// Timeslice for the next order was set.
		NextOrderScheduled { next_order: Timeslice },
		/// Coretime requirements got set.
		///
		/// If `None` it means that the parachain will stop with Coretime procurement.
		CoretimeRequirementSet { requirements: Option<GenericRequirements> },
	}

	#[pallet::error]
	#[derive(PartialEq)]
	pub enum Error<T> {}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(_now: BlockNumberFor<T>) -> Weight {
			let mut weight = T::DbWeight::get().reads(1);

			let Some(config) = Configuration::<T>::get() else {
				log::warn!(
					target: LOG_TARGET,
					"Coretime chain configuration not set",
				);
				return weight;
			};

			weight += T::DbWeight::get().reads(1);
			let Some(current_order) = NextOrder::<T>::get() else {
				log::warn!(
					target: LOG_TARGET,
					"The timeslice for the next order not set",
				);
				return weight;
			};

			if Self::current_timeslice() >= current_order {
				weight += T::DbWeight::get().reads(1);
				let Some(generic) = CoretimeRequirements::<T>::get() else {
					log::warn!(
						target: LOG_TARGET,
						"The coretime requirements are not set",
					);
					return weight;
				};

				let requirements = OrderRequirements {
					begin: current_order,
					end: current_order.saturating_add(config.region_length),
					core_occupancy: generic.core_occupancy,
				};
				if let Err(e) = T::OrderDispatcher::dispatch(requirements) {
					log::error!(
						target: LOG_TARGET,
						"Failed to dispatch order: {:?}",
						e
					);
				}
				// TODO: better naming for 'current_order'.
				NextOrder::<T>::set(Some(current_order.saturating_add(config.region_length)));
				// TODO: account for the dispatcher weight consumption:
				weight
			} else {
				weight
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Set the configuration of the Coretime chain.
		///
		/// - `origin`: Must be Root or pass `AdminOrigin`.
		/// - `configuration`: The configuration of the Coretime chain.
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::set_configuration())]
		pub fn set_configuration(
			origin: OriginFor<T>,
			configuration: ConfigRecordOf<T>,
		) -> DispatchResult {
			T::AdminOrigin::ensure_origin_or_root(origin)?;

			Configuration::<T>::put(configuration.clone());
			Self::deposit_event(Event::ConfigurationSet { configuration });
			Ok(())
		}

		/// Set the timeslice at which we create the next order.
		///
		/// - `origin`: Must be Root or pass `AdminOrigin`.
		/// - `next_order`: The timeslice at which to create the next order.
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::schedule_next_order())]
		pub fn schedule_next_order(origin: OriginFor<T>, next_order: Timeslice) -> DispatchResult {
			T::AdminOrigin::ensure_origin_or_root(origin)?;

			NextOrder::<T>::put(next_order);

			Self::deposit_event(Event::NextOrderScheduled { next_order });
			Ok(())
		}

		/// Extrinsic for creating an order.
		///
		/// ## Arguments:
		/// - `requirements`: The coretime requirements of the parachain. If set to `None` the
		///   pallet will stop with the order creation.
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::set_coretime_requirements())]
		pub fn set_coretime_requirements(
			origin: OriginFor<T>,
			requirements: Option<GenericRequirements>,
		) -> DispatchResult {
			T::AdminOrigin::ensure_origin_or_root(origin)?;

			CoretimeRequirements::<T>::set(requirements.clone());

			Self::deposit_event(Event::CoretimeRequirementSet { requirements });
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub(crate) fn current_timeslice() -> Timeslice {
			let latest_rc_block = T::RCBlockNumberProvider::current_block_number();
			let timeslice_period = T::TimeslicePeriod::get();
			(latest_rc_block / timeslice_period).saturated_into()
		}
	}
}
