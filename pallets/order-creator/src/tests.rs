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

use frame_support::{assert_noop, assert_ok};
use sp_core::Get;
use sp_runtime::{traits::BadOrigin, Perbill};

use crate::{
	mock::*, Config, ConfigRecordOf, Configuration, CoretimeRequirements, Event,
	GenericRequirements, NextOrder,
};

#[test]
fn set_configuration_works() {
	new_test_ext().execute_with(|| {
		assert!(Configuration::<Test>::get().is_none());

		let configuration = ConfigRecordOf::<Test> {
			advance_notice: 10,
			interlude_length: 7_200,
			leadin_length: 21_600,
			region_length: 1_260,
			ideal_bulk_proportion: Perbill::from_percent(40),
			limit_cores_offered: None,
			renewal_bump: Perbill::from_percent(40),
			contribution_timeout: 1_260,
		};
		// Failure: Bad origin
		assert_noop!(
			OrderCreator::set_configuration(RuntimeOrigin::signed(ALICE), configuration.clone()),
			BadOrigin
		);

		// Should be working fine
		assert_ok!(OrderCreator::set_configuration(RuntimeOrigin::root(), configuration.clone()));

		// Check the storage item
		assert_eq!(Configuration::<Test>::get(), Some(configuration.clone()));

		// Check the emitted events
		System::assert_last_event(Event::ConfigurationSet { configuration }.into());
	})
}

#[test]
fn schedule_next_order_works() {
	new_test_ext().execute_with(|| {
		assert!(NextOrder::<Test>::get().is_none());

		// Failure: Bad origin
		assert_noop!(OrderCreator::schedule_next_order(RuntimeOrigin::signed(ALICE), 1), BadOrigin);

		// Should work
		assert_ok!(OrderCreator::schedule_next_order(RuntimeOrigin::root(), 1));

		// Check the storage item
		assert_eq!(NextOrder::<Test>::get(), Some(1));

		// Check the emitted events
		System::assert_last_event(Event::NextOrderScheduled { next_order: 1 }.into());
	});
}

#[test]
fn set_coretime_requirements_works() {
	new_test_ext().execute_with(|| {
		assert!(CoretimeRequirements::<Test>::get().is_none());

		// Failure: Bad Origin
		assert_noop!(
			OrderCreator::set_coretime_requirements(RuntimeOrigin::signed(ALICE), None),
			BadOrigin
		);

		let requirements = Some(GenericRequirements { core_occupancy: 28_800 }); // 50%

		// Should work
		assert_ok!(OrderCreator::set_coretime_requirements(
			RuntimeOrigin::root(),
			requirements.clone()
		));

		// Check the storage item
		assert_eq!(CoretimeRequirements::<Test>::get(), requirements.clone());

		// Check the emitted events
		System::assert_last_event(Event::CoretimeRequirementSet { requirements }.into())
	});
}

#[test]
fn current_timeslice_works() {
	new_test_ext().execute_with(|| {
		RelayBlockNumber::set(0);
		let timeslice_period: u64 = <Test as Config>::TimeslicePeriod::get();

		assert_eq!(OrderCreator::current_timeslice(), 0);

		RelayBlockNumber::set(5 * timeslice_period);
		assert_eq!(OrderCreator::current_timeslice(), 5);

		RelayBlockNumber::set(6 * timeslice_period - 1);
		assert_eq!(OrderCreator::current_timeslice(), 5);
	});
}
