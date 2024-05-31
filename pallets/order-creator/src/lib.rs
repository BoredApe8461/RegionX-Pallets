//! Order creator pallet.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::Currency;
use frame_system::WeightInfo;
pub use pallet::*;

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

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {}

	#[pallet::error]
	#[derive(PartialEq)]
	pub enum Error<T> {}

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

		/// Extrinsic for creating an order.
		///
		/// ## Arguments:
		/// - `requirements`: The coretime requirements of the parachain. If set to `None` the
		///   pallet will stop with the order creation.
		#[pallet::call_index(1)]
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
}
