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

use crate::{Pallet as Seasons, *};
use ajuna_primitives::season_manager::SeasonFeeConfig;

use frame_benchmarking::v2::*;
use frame_support::BoundedVec;
use frame_system::RawOrigin;
use sp_runtime::Saturating;

const ACC_1: &str = "acc_1";

fn account<T: Config<I>, I: 'static>(name: &'static str) -> T::AccountId {
	let index = 0;
	let seed = 0;
	frame_benchmarking::account(name, index, seed)
}

fn assert_last_event<T: Config<I>, I: 'static>(avatars_event: Event<T, I>) {
	let event = <T as frame_system::Config>::RuntimeEvent::from(avatars_event);
	frame_system::Pallet::<T>::assert_last_event(event);
}

fn setup_organizer<T: Config<I>, I: 'static>(organizer: T::AccountId) {
	T::AccountHandler::set_organizer(organizer);
}

fn run_to_block<T: Config<I>, I: 'static>(n: BlockNumberFor<T>) {
	while frame_system::Pallet::<T>::block_number() < n {
		let mut current_block = frame_system::Pallet::<T>::block_number();
		if current_block > 1_u32.into() {
			frame_system::Pallet::<T>::on_finalize(current_block);
			crate::Pallet::<T, I>::on_finalize(current_block);
		}
		current_block = current_block.saturating_add(1_u32.into());
		frame_system::Pallet::<T>::set_block_number(current_block);
		frame_system::Pallet::<T>::on_initialize(current_block);
		crate::Pallet::<T, I>::on_initialize(current_block);
	}
}

#[instance_benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn update_season() {
		let season_id = T::BenchmarkHelper::create_season_id(1_u32);
		let acc_1 = account::<T, I>(ACC_1);
		setup_organizer::<T, I>(acc_1.clone());
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
		};
		let metadata = SeasonMetadata {
			name: BoundedVec::try_from(b"Season-1".to_vec()).expect("Should create vec"),
			description: BoundedVec::try_from(b"The first season".to_vec())
				.expect("Should create vec"),
		};
		let schedule = SeasonSchedule {
			early_start: 20_u32.into(),
			start: 25_u32.into(),
			end: Some(30_u32.into()),
		};

		#[extrinsic_call]
		_(
			RawOrigin::Signed(acc_1),
			season_id.clone(),
			Some(config.clone()),
			Some(metadata.clone()),
			Some(schedule.clone()),
		);

		assert_last_event::<T, I>(Event::UpdatedSeason {
			season_id,
			config: Some(config),
			metadata: Some(metadata),
			schedule: Some(schedule),
		});
	}

	#[benchmark]
	fn interrupt_active_season() {
		let season_id = T::BenchmarkHelper::create_season_id(2_u32);
		let acc_1 = account::<T, I>(ACC_1);
		setup_organizer::<T, I>(acc_1.clone());
		let config = SeasonConfigOf::<T, I> {
			fee: SeasonFeeConfig {
				transfer_asset: 20_u32.into(),
				buy_asset_min: 5_u32.into(),
				buy_percent: 10,
				upgrade_asset_inventory: 5_u32.into(),
				unlock_trade_asset: 9_u32.into(),
				unlock_transfer_asset: 13_u32.into(),
				state_transition_base_fee: 20_u32.into(),
			},
		};
		let metadata = SeasonMetadata {
			name: BoundedVec::try_from(b"Season-1".to_vec()).expect("Should create vec"),
			description: BoundedVec::try_from(b"The first season".to_vec())
				.expect("Should create vec"),
		};
		let schedule = SeasonSchedule {
			early_start: 20_u32.into(),
			start: 25_u32.into(),
			end: Some(30_u32.into()),
		};
		Seasons::<T, I>::update_season(
			RawOrigin::Signed(acc_1.clone()).into(),
			season_id.clone(),
			Some(config),
			Some(metadata),
			Some(schedule),
		)
		.expect("Should update season");
		run_to_block::<T, I>(25_u32.into());

		#[extrinsic_call]
		_(RawOrigin::Signed(acc_1));

		assert_last_event::<T, I>(Event::SeasonEarlyEnded { season_id });
	}

	impl_benchmark_test_suite!(Seasons, crate::mock::new_test_ext(), crate::mock::Test);
}
