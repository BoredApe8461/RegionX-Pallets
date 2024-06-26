// This file is part of RegionX.
//
// RegionX is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// RegionX is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with RegionX.  If not, see <https://www.gnu.org/licenses/>.

//! Benchmarks for pallet-order-creator

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::v2::*;
use frame_support::traits::Get;
use frame_system::RawOrigin;
fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_broker::ConfigRecord;

#[benchmarks]
mod benchmarks {
	use super::*;
	use frame_support::traits::EnsureOrigin;
	use sp_runtime::Perbill;

	#[benchmark]
	fn set_configuration() -> Result<(), BenchmarkError> {
		let origin =
			T::AdminOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;

		let configuration = ConfigRecord::<BlockNumberFor<T>, RCBlockNumberOf<T>> {
			advance_notice: 10u32.into(),
			interlude_length: 7_200u32.into(),
			leadin_length: 21_600u32.into(),
			region_length: 1_260u32.into(),
			ideal_bulk_proportion: Perbill::from_percent(40),
			limit_cores_offered: None,
			renewal_bump: Perbill::from_percent(40),
			contribution_timeout: 1_260u32.into(),
		};

		#[extrinsic_call]
		_(origin as T::RuntimeOrigin, configuration.clone());

		assert_last_event::<T>(Event::ConfigurationSet { configuration }.into());
		Ok(())
	}

	#[benchmark]
	fn schedule_next_order() -> Result<(), BenchmarkError> {
		let origin =
			T::AdminOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		let next_order = 100u32.into();

		#[extrinsic_call]
		_(origin as T::RuntimeOrigin, next_order);

		assert_last_event::<T>(Event::NextOrderScheduled { next_order }.into());
		Ok(())
	}

	#[benchmark]
	fn set_coretime_requirements() -> Result<(), BenchmarkError> {
		let origin =
			T::AdminOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		let requirements = Some(GenericRequirements { core_occupancy: 28800u16.into() });

		#[extrinsic_call]
		_(origin as T::RuntimeOrigin, requirements.clone());

		assert_last_event::<T>(Event::CoretimeRequirementSet { requirements }.into());
		Ok(())
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
