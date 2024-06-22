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

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn set_configuration() -> Result<(), BenchmarkError> {
		let configuration = ConfigRecord {
			advance_notice: 10,
			interlude_length: 7_200,
			leadin_length: 21_600,
			region_length: 1_260,
			ideal_bulk_proportion: Perbill::from_percent(40),
			limit_cores_offered: None,
			renewal_bump: Perbill::from_percent(40),
			contribution_timeout: 1_260,
		};

		#[extrinsic_call]
		_(RuntimeOrigin::root(), configuration.clone());

		assert_last_event::<T>(Event::ConfigurationSet { configuration }.into());
		Ok(())
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(vec![]), crate::mock::Test);
}
