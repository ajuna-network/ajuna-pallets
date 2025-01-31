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

use crate::{mock::*, tests::*, *};
use ajuna_primitives::season_manager::SeasonFeeConfig;
use frame_support::{assert_noop, assert_ok};

fn start_season(organizer: MockAccountId, season_id: MockSeasonId) {
	let config = SeasonConfigOf::<Test, _> {
		fee: SeasonFeeConfig {
			transfer_asset: 10_u64,
			buy_asset_min: 5_u64,
			buy_percent: 10,
			upgrade_asset_inventory: 5_u64,
			unlock_trade_asset: 9_u64,
			unlock_transfer_asset: 13_u64,
			state_transition_base_fee: 20_u64,
		},
		data: MockSeasonData { data: 24 },
	};
	let early_start = 20;
	let schedule = SeasonSchedule { early_start, start: 25, end: 30 };

	assert_ok!(SeasonsAlpha::update_season(
		RuntimeOrigin::signed(organizer),
		season_id,
		Some(config),
		None,
		Some(schedule),
	));

	run_to_block(early_start);
}

mod season_manager {
	use super::*;

	#[test]
	fn get_season_id_for_works() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			start_season(ALICE, SEASON_ID_1);

			let asset_id = MockAssetId::from(10_u32);
			assert_ok!(<SeasonsAlpha as SeasonManager>::register_asset_in(&asset_id, &SEASON_ID_1));

			assert_eq!(
				<SeasonsAlpha as SeasonManager>::get_season_id_for(&asset_id),
				Ok(SEASON_ID_1)
			);
		});
	}

	#[test]
	fn get_current_season_id_works_if_active_season() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			start_season(ALICE, SEASON_ID_2);

			System::assert_last_event(RuntimeEvent::SeasonsAlpha(Event::SeasonEarlyStarted {
				season_id: SEASON_ID_2,
			}));

			assert_eq!(<SeasonsAlpha as SeasonManager>::get_current_season_id(), Ok(SEASON_ID_2));
		});
	}

	#[test]
	fn get_current_season_id_rejects_if_no_active_season() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			assert_noop!(
				<SeasonsAlpha as SeasonManager>::get_current_season_id(),
				Error::<Test, _>::NoActiveSeason
			);
		});
	}

	#[test]
	fn is_valid_season_works() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			start_season(ALICE, SEASON_ID_2);

			assert_ok!(<SeasonsAlpha as SeasonManager>::is_valid_season(&SEASON_ID_2));
			assert_noop!(
				<SeasonsAlpha as SeasonManager>::is_valid_season(&SEASON_ID_1),
				Error::<Test, _>::InvalidSeason
			);
		});
	}

	#[test]
	fn get_season_config_for_works() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			let config = SeasonConfigOf::<Test, _> {
				fee: SeasonFeeConfig {
					transfer_asset: 10_u64,
					buy_asset_min: 5_u64,
					buy_percent: 10,
					upgrade_asset_inventory: 5_u64,
					unlock_trade_asset: 9_u64,
					unlock_transfer_asset: 13_u64,
					state_transition_base_fee: 20_u64,
				},
				data: MockSeasonData { data: 24 },
			};
			let schedule = SeasonSchedule { early_start: 20, start: 25, end: 30 };

			assert_ok!(SeasonsAlpha::update_season(
				RuntimeOrigin::signed(ALICE),
				SEASON_ID_1,
				Some(config.clone()),
				None,
				Some(schedule)
			));

			assert_eq!(
				<SeasonsAlpha as SeasonManager>::get_season_config_for(&SEASON_ID_1),
				Ok(config)
			);
		});
	}

	#[test]
	fn get_season_config_for_rejects_for_invalid_season() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			assert_noop!(
				<SeasonsAlpha as SeasonManager>::get_season_config_for(&SEASON_ID_1),
				Error::<Test, _>::InvalidSeason
			);
		});
	}

	#[test]
	fn register_asset_in_works() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			start_season(ALICE, SEASON_ID_1);

			let asset_id = MockAssetId::from(612_u32);
			assert_eq!(AssetSeasonRegister::<Test, _>::get(asset_id), None);

			assert_ok!(<SeasonsAlpha as SeasonManager>::register_asset_in(&asset_id, &SEASON_ID_1));

			assert_eq!(AssetSeasonRegister::<Test, _>::get(asset_id), Some(SEASON_ID_1));
		});
	}

	#[test]
	fn register_asset_in_rejects_registering_asset_to_invalid_season() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			let asset_id = MockAssetId::from(43_u32);
			assert_noop!(
				<SeasonsAlpha as SeasonManager>::register_asset_in(&asset_id, &SEASON_ID_1),
				Error::<Test, _>::InvalidSeason
			);
		});
	}
}
