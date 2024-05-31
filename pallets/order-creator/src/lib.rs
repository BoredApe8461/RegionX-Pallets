//! Order creator pallet.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::Currency;
use frame_system::WeightInfo;
pub use pallet::*;
use xcm_executor::traits::ConvertLocation;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod types;
pub use crate::types::*;

pub type BalanceOf<T> =
	<<T as crate::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		pallet_prelude::*,
		sp_runtime::Saturating,
		traits::{fungible::Transfer, Get, ReservableCurrency},
	};
	use frame_system::pallet_prelude::*;

	/// The module configuration trait.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The relay chain currency used for coretime procurement.
		type RelaychainCurrency: Transfer<Self::AccountId>;

        // TODO: this is not for this pallet, this is for the renewal financing. just noting this here so I don't forget the idea.
        //
        // The expense cap will ensure that the RegionX chain can't spam new sale notifications.
        type ExpenseCap;

		/// Weight Info
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Created orders.
	#[pallet::storage]
	#[pallet::getter(fn coretime_requirement)]
	pub type CoretimeRequirement<T: Config> = StorageValue<_, Requirements, OptionQuery>;

    
	/// Next order id
	#[pallet::storage]
	#[pallet::getter(fn next_order_id)]
	pub type NextOrderId<T> = StorageValue<_, OrderId, ValueQuery>;

	/// Crowdfunding contributions.
	#[pallet::storage]
	#[pallet::getter(fn contributions)]
	pub type Contributions<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		OrderId, // the order.
		Blake2_128Concat,
		T::AccountId, // the account which contributed to the order.
		BalanceOf<T>, // the amount they contributed.
		ValueQuery,
	>;

	/// The total amount that was contributed to an order.
	///
	/// The sum of contributions for a specific order from the Contributions map should be equal to
	/// the total contribution stored here.
	#[pallet::storage]
	#[pallet::getter(fn total_contributions)]
	pub type TotalContributions<T: Config> =
		StorageMap<_, Blake2_128Concat, OrderId, BalanceOf<T>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {}

	#[pallet::error]
	#[derive(PartialEq)]
	pub enum Error<T> {
		/// Invalid order id
		InvalidOrderId,
		/// The caller is not the creator of the given order
		NotAllowed,
		/// The contribution amount is too small
		InvalidAmount,
		/// The given order is not cancelled
		OrderNotCancelled,
		/// The contributed amount equals to zero.
		NoContribution,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Extrinsic for creating an order.
		///
		/// ## Arguments:
		/// - `para_id`: The para id to which Coretime will be allocated.
		/// - `requirements`: Region requirements of the order.
		#[pallet::call_index(0)]
		#[pallet::weight(10_000)] // TODO
		pub fn create_order(
			origin: OriginFor<T>,
			para_id: ParaId,
			requirements: Requirements,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			T::OrderCreationFeeHandler::handle(&who, T::OrderCreationCost::get())?;

			let order_id = NextOrderId::<T>::get();
			Orders::<T>::insert(order_id, Order { creator: who, para_id, requirements });
			NextOrderId::<T>::put(order_id + 1);

			Self::deposit_event(Event::OrderCreated { order_id });
			Ok(())
		}

		/// Extrinsic for cancelling an order.
		///
		/// ## Arguments:
		/// - `para_id`: The para id to which Coretime will be allocated.
		/// - `requirements`: Region requirements of the order.
		#[pallet::call_index(1)]
		#[pallet::weight(10_000)] // TODO
		pub fn cancel_order(origin: OriginFor<T>, order_id: OrderId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let order = Orders::<T>::get(order_id).ok_or(Error::<T>::InvalidOrderId)?;
			ensure!(order.creator == who, Error::<T>::NotAllowed);

			Orders::<T>::remove(order_id);

			Self::deposit_event(Event::OrderRemoved { order_id });
			Ok(())
		}

		/// Extrinsic for contributing to an order.
		///
		/// ## Arguments:
		/// - `order_id`: The order to which the caller wants to contribute.
		/// - `amount`: The amount of tokens the user wants to contribute.
		#[pallet::call_index(2)]
		#[pallet::weight(10_000)] // TODO
		pub fn contribute(
			origin: OriginFor<T>,
			order_id: OrderId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(Orders::<T>::get(order_id).is_some(), Error::<T>::InvalidOrderId);
			ensure!(amount >= T::MinimumContribution::get(), Error::<T>::InvalidAmount);
			T::Currency::reserve(&who, amount)?;

			let mut contribution: BalanceOf<T> = Contributions::<T>::get(order_id, who.clone());

			contribution = contribution.saturating_add(amount);
			Contributions::<T>::insert(order_id, who.clone(), contribution);

			let mut total_contributions = TotalContributions::<T>::get(order_id);

			total_contributions = total_contributions.saturating_add(amount);
			TotalContributions::<T>::insert(order_id, total_contributions);

			Self::deposit_event(Event::Contributed { order_id, who, amount });

			Ok(())
		}

		/// Extrinsic for removing contributions from a cancelled order.
		///
		/// ## Arguments:
		/// - `order_id`: The cancelled order from which the user wants to claim back their
		///   contribution.
		#[pallet::call_index(3)]
		#[pallet::weight(10_000)] // TODO
		pub fn remove_contribution(origin: OriginFor<T>, order_id: OrderId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(Orders::<T>::get(order_id).is_none(), Error::<T>::OrderNotCancelled);

			let amount: BalanceOf<T> = Contributions::<T>::get(order_id, who.clone());
			ensure!(amount != Default::default(), Error::<T>::NoContribution);

			T::Currency::unreserve(&who, amount);
			Contributions::<T>::remove(order_id, who.clone());

			let mut total_contributions = TotalContributions::<T>::get(order_id);

			total_contributions = total_contributions.saturating_sub(amount);
			TotalContributions::<T>::insert(order_id, total_contributions);

			Self::deposit_event(Event::ContributionRemoved { who, order_id, amount });

			Ok(())
		}
	}
}
