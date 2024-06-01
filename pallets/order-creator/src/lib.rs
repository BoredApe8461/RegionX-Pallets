//! Order creator pallet.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::Currency;
use frame_system::WeightInfo;
pub use pallet::*;
use pallet_broker::Timeslice;
use sp_runtime::SaturatedConversion;

mod types;
pub use crate::types::*;

pub type BalanceOf<T> = <<T as crate::Config>::RelaychainCurrency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;

const LOG_TARGET: &str = "runtime::order-creator";

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		pallet_prelude::*,
		traits::{fungible::Mutate, Get},
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::BlockNumberProvider;

	/// The module configuration trait.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The relay chain currency used for coretime procurement.
		type RelaychainCurrency: Mutate<Self::AccountId>;

		/// A means of getting the current relay chain block.
		///
		/// This is used for determining the current timeslice.
		type RCBlockNumberProvider: BlockNumberProvider;

		/// The admin origin for managing the order creation.
		type AdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// Number of Relay-chain blocks per timeslice.
		#[pallet::constant]
		type TimeslicePeriod: Get<RCBlockNumberOf<Self>>;

		/// Weight Info
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// The coretime requirements for the parachain.
	///
	/// Orders will be made based on this.
	#[pallet::storage]
	#[pallet::getter(fn coretime_requirements)]
	pub type CoretimeRequirements<T: Config> = StorageValue<_, Requirements, OptionQuery>;

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

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {}

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
			let Some(next_order) = NextOrder::<T>::get() else {
				log::warn!(
					target: LOG_TARGET,
					"The timeslice for the next order not set",
				);
				return weight;
			};

			if Self::current_timeslice() >= next_order {
				// TODO: create order
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
		/// - `config`: The configuration the Coretime chain.
		#[pallet::call_index(0)]
		#[pallet::weight(10_000)] // TODO
		pub fn set_configuration(
			origin: OriginFor<T>,
			config: ConfigRecordOf<T>,
		) -> DispatchResult {
			T::AdminOrigin::ensure_origin_or_root(origin)?;

			Configuration::<T>::put(config);
			// TODO: event

			Ok(())
		}

		/// Set the timeslice at which we create the next order.
		///
		/// - `origin`: Must be Root or pass `AdminOrigin`.
		/// - `next_order`: The timeslice at which to create the next order.
		#[pallet::call_index(1)]
		#[pallet::weight(10_000)] // TODO
		pub fn set_next_order(origin: OriginFor<T>, next_order: Timeslice) -> DispatchResult {
			T::AdminOrigin::ensure_origin_or_root(origin)?;

			NextOrder::<T>::put(next_order);

			// TODO: event

			Ok(())
		}

		/// Extrinsic for creating an order.
		///
		/// ## Arguments:
		/// - `requirements`: The coretime requirements of the parachain. If set to `None` the
		///   pallet will stop with the order creation.
		#[pallet::call_index(2)]
		#[pallet::weight(10_000)] // TODO
		pub fn set_coretime_requirements(
			origin: OriginFor<T>,
			requirements: Option<Requirements>,
		) -> DispatchResult {
			T::AdminOrigin::ensure_origin_or_root(origin)?;

			CoretimeRequirements::<T>::set(requirements);
			// TODO: event

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
