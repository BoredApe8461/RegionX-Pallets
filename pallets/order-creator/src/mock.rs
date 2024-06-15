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

use crate::ParaId;
use codec::{Decode, Encode};
use frame_support::{
	pallet_prelude::*,
	parameter_types,
	traits::Everything,
	weights::{
		constants::ExtrinsicBaseWeight, WeightToFeeCoefficient, WeightToFeeCoefficients,
		WeightToFeePolynomial,
	},
};
use frame_system::EnsureRoot;
use smallvec::smallvec;
use sp_core::{ConstU64, H256};
use sp_runtime::{
	traits::{BlakeTwo256, BlockNumberProvider, IdentityLookup},
	BuildStorage, Perbill,
};
use xcm::latest::prelude::*;

type Block = frame_system::mocking::MockBlock<Test>;
type AccountId = u64;

pub const ALICE: AccountId = 1;

frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
		Balances: pallet_balances,
		OrderCreator: crate::{Pallet, Call, Storage, Event<T>}
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
	pub const AnyNetwork: Option<NetworkId> = None;
}

impl frame_system::Config for Test {
	type BaseCallFilter = Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Nonce = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeTask = RuntimeTask;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_balances::Config for Test {
	type Balance = u64;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ConstU64<1>;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxHolds = ();
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = [u8; 8];
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type FreezeIdentifier = ();
	type MaxFreezes = ();
}

pub const MILLIUNIT: u64 = 1_000_000_000;
pub struct WeightToFee;
impl WeightToFeePolynomial for WeightToFee {
	type Balance = u64;
	fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
		// in Rococo, extrinsic base weight (smallest non-zero weight) is mapped to 1 MILLIUNIT:
		// in our template, we map to 1/10 of that, or 1/10 MILLIUNIT
		let p = MILLIUNIT / 10;
		let q = 100 * u64::from(ExtrinsicBaseWeight::get().ref_time());
		smallvec![WeightToFeeCoefficient {
			degree: 1,
			negative: false,
			coeff_frac: Perbill::from_rational(p % q, q),
			coeff_integer: p / q,
		}]
	}
}

parameter_types! {
	pub static RelayBlockNumber: u64 = 0;
	// The location of the RegionX parachain.
	pub const RegionXLocation: MultiLocation = MultiLocation { parents: 1, interior: X1(Parachain(2000)) };
}

pub struct RelayBlockNumberProvider;
impl BlockNumberProvider for RelayBlockNumberProvider {
	type BlockNumber = u64;
	fn current_block_number() -> Self::BlockNumber {
		RelayBlockNumber::get()
	}
}

use crate::OrderRequirements;

#[derive(Encode, Decode)]
enum RegionXRuntimeCalls {
	#[codec(index = 92)]
	Orders(OrderPalletCalls),
}

/// RegionX coretime pallet calls.
//
// NOTE: We only use the `CreateOrder` call.
#[derive(Encode, Decode)]
enum OrderPalletCalls {
	#[codec(index = 0)]
	CreateOrder(ParaId, OrderRequirements),
}

pub struct CallEncoder;
impl crate::CallEncoder for CallEncoder {
	fn order_creation_call(requirements: OrderRequirements) -> Vec<u8> {
		RegionXRuntimeCalls::Orders(OrderPalletCalls::CreateOrder(
			2001.into(), // dummy para id.
			requirements,
		))
		.encode()
	}
}

pub struct DummyOrderDispatcher;
impl crate::OrderDispatcher for DummyOrderDispatcher {
	fn dispatch(_requirements: OrderRequirements) -> DispatchResult {
		Ok(())
	}
}

impl crate::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type RelaychainCurrency = Balances;
	type RelaychainBalance = u64;
	type RCBlockNumberProvider = RelayBlockNumberProvider;
	type RegionXLocation = RegionXLocation;
	type AdminOrigin = EnsureRoot<<Test as frame_system::Config>::AccountId>;
	type OrderDispatcher = DummyOrderDispatcher;
	type CallEncoder = CallEncoder;
	type WeightToFee = WeightToFee;
	type TimeslicePeriod = ConstU64<80>;
	type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}
