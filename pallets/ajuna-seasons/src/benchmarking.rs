// Ajuna Node
// Copyright (C) 2022 BlogaTech AG

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

#![cfg(feature = "runtime-benchmarks")]
#![cfg_attr(not(feature = "std"), no_std)]

use crate::{
	mock::{
		run_to_block, Balances, MockAccountManager, MockAssetId, MockSeasonData, MockSeasonId,
		RuntimeEvent, SeasonsBenchmarkHelper, System, Test,
	},
	Pallet as Seasons, *,
};
use ajuna_primitives::season_manager::SeasonFeeConfig;

use frame_benchmarking::benchmarks_instance_pallet;
use frame_support::BoundedVec;
use frame_system::RawOrigin;
use sp_runtime::BuildStorage;

impl Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type SeasonId = MockSeasonId;
	type SeasonData = MockSeasonData;
	type AssetId = MockAssetId;
	type AccountHandler = MockAccountManager;
	type Currency = Balances;
	type WeightInfo = ();
	type BenchmarkHelper = SeasonsBenchmarkHelper;
}

const ACC_1: &str = "acc_1";

fn account<T: Config<I>, I: 'static>(name: &'static str) -> T::AccountId {
	let index = 0;
	let seed = 0;
	frame_benchmarking::account(name, index, seed)
}

fn assert_last_event<T: Config<I>, I: 'static>(avatars_event: Event<T, I>) {
	let event = <T as Config<I>>::RuntimeEvent::from(avatars_event);
	frame_system::Pallet::<T>::assert_last_event(event.into());
}

benchmarks_instance_pallet! {
	update_season {
		let season_id = T::BenchmarkHelper::create_season_id(1_u32);
		let acc_1 = account::<T, I>(ACC_1);
		let config = SeasonConfigOf::<T, I> {
				fee: SeasonFeeConfig {
					transfer_asset: 10_u32.into(),
					buy_asset_min: 5_u32.into(),
					buy_percent: 10,
					upgrade_asset_inventory: 5_u32.into(),
					unlock_trade_asset: 9_u32.into(),
					unlock_transfer_asset: 13_u32.into(),
					state_transition_base_fee: 20_u32.into(),
				},
				data: T::BenchmarkHelper::create_default_season_data(),
			};
		let metadata = SeasonMetadata {
			name: BoundedVec::try_from(b"Season-1".to_vec()).expect("Should create vec"),
			description: BoundedVec::try_from(b"The first season".to_vec())
				.expect("Should create vec"),
		};
		let schedule = SeasonSchedule {
			early_start: 20_u32.into(),
			start: 25_u32.into(),
			end: 30_u32.into()
		};
	}: _(RawOrigin::Signed(acc_1), season_id.clone(), Some(config.clone()), Some(metadata.clone()), Some(schedule.clone()))
	verify {
		assert_last_event::<T, I>(Event::UpdatedSeason {
			season_id,
			config: Some(config),
			metadata: Some(metadata),
			schedule: Some(schedule)
		})
	}

	interrupt_active_season {
		let season_id = T::BenchmarkHelper::create_season_id(2_u32);
		let acc_1 = account::<T, I>(ACC_1);
		let config = SeasonConfigOf::<T, I> {
				fee: SeasonFeeConfig {
					transfer_asset: 10_u32.into(),
					buy_asset_min: 5_u32.into(),
					buy_percent: 10,
					upgrade_asset_inventory: 5_u32.into(),
					unlock_trade_asset: 9_u32.into(),
					unlock_transfer_asset: 13_u32.into(),
					state_transition_base_fee: 20_u32.into(),
				},
				data: T::BenchmarkHelper::create_default_season_data(),
			};
		let metadata = SeasonMetadata {
			name: BoundedVec::try_from(b"Season-1".to_vec()).expect("Should create vec"),
			description: BoundedVec::try_from(b"The first season".to_vec())
				.expect("Should create vec"),
		};
		let schedule = SeasonSchedule {
			early_start: 20_u32.into(),
			start: 25_u32.into(),
			end: 30_u32.into()
		};
		Seasons::<T, I>::update_season(
			RawOrigin::Signed(acc_1.clone()).into(),
			season_id.clone(),
			Some(config),
			Some(metadata),
			Some(schedule)
		).expect("Should update season");
		run_to_block(25_u32.into());
	}: _(RawOrigin::Signed(acc_1))
	verify {
		assert_last_event::<T, I>(Event::SeasonEarlyEnded {
			season_id,
		})
	}

	impl_benchmark_test_suite!(
		Seasons,
		new_test_ext(),
		Test
	);
}

#[allow(dead_code)]
pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext.execute_with(|| {
		let acc_1 = account::<Test, ()>(ACC_1);
		MockAccountManager::set_organizer(acc_1);
	});
	ext
}
