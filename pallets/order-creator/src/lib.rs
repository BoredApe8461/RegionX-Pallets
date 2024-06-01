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
	#[pallet::storage]
	#[pallet::getter(fn configuration)]
	pub type Configuration<T: Config> = StorageValue<_, ConfigRecordOf<T>, OptionQuery>;

	/// The start of the latest bulk period.
	///
	/// When setting up the pallet, this should be set to the start of the latest bulk period by the
	/// `AdminOrigin`.
	///
	/// After that, the pallet will keep this up to date by itself.
	///
	/// *WARNING*: If the sale duration changes on the Coretime chain, the `AdminOrigin` should
	/// update the `Configuration` to match the new config on the Coretime chain. If not updated, we
	/// run the risk of creating redundant orders or missing an order creation.
	///
	/// Ideally, we would read the current sale state directly from the Coretime chain. However,
	/// that would require using something like ISMP and a relay infrastructure.
	#[pallet::storage]
	#[pallet::getter(fn bulk_period_start)]
	pub type BulkPeriodStart<T: Config> = StorageValue<_, Timeslice, OptionQuery>;

	/// The timeslice when the last order was made.
	#[pallet::storage]
	#[pallet::getter(fn last_order)]
	pub type LastOrder<T: Config> = StorageValue<_, Timeslice, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {}

	#[pallet::error]
	#[derive(PartialEq)]
	pub enum Error<T> {
		/// The `AdminOrigin` provided an invalid bulk period start.
		InvalidBulkPeriodStart,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(_now: BlockNumberFor<T>) -> Weight {
			let last_order = LastOrder::<T>::get();
			let config = Configuration::<T>::get();

			//let current_sale_start = ReferenceSaleStart::<T>::get().saturating_add(config.)

			Weight::zero()
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

		/// Set the start of the latest bulk period.
		///
		/// - `origin`: Must be Root or pass `AdminOrigin`.
		/// - `bulk_period_start`: The start of the latest bulk period.
		#[pallet::call_index(1)]
		#[pallet::weight(10_000)] // TODO
		pub fn set_bulk_period_start(
			origin: OriginFor<T>,
			bulk_period_start: Timeslice,
		) -> DispatchResult {
			T::AdminOrigin::ensure_origin_or_root(origin)?;

			// Sanity check:
			ensure!(
				bulk_period_start <= Self::current_timeslice(),
				Error::<T>::InvalidBulkPeriodStart
			);
			BulkPeriodStart::<T>::put(bulk_period_start);

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
