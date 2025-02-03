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

use crate::{mock::*, *};
use ajuna_primitives::season_manager::SeasonFeeConfig;
use frame_support::{assert_err, assert_noop, assert_ok};

pub(crate) const SEASON_ID_1: MockSeasonId = 1;
pub(crate) const SEASON_ID_2: MockSeasonId = 2;

pub(crate) const ALICE: MockAccountId = MockAccountId::new([1; 32]);
pub(crate) const BOB: MockAccountId = MockAccountId::new([2; 32]);

#[test]
fn test_seasons_full_workflow() {
	ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
		run_to_block(10);

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
		};
		let metadata = SeasonMetadata {
			name: BoundedVec::try_from(b"Season-1".to_vec()).expect("Should create vec"),
			description: BoundedVec::try_from(b"The first season".to_vec())
				.expect("Should create vec"),
		};
		let schedule = SeasonSchedule { early_start: 20, start: 25, end: Some(30) };

		// Initially there is no data in storage
		assert_err!(CurrentSeasonStatus::<Test, _>::get(), Error::<Test, _>::NoActiveSeason);
		assert_eq!(LatestSeason::<Test, _>::get(), None);
		assert_eq!(FinishedSeasons::<Test, _>::iter().count(), 0);
		assert_eq!(NextSeasonChain::<Test, _>::iter().count(), 0);
		assert_eq!(PrevSeasonChain::<Test, _>::iter().count(), 0);
		assert_eq!(Seasons::<Test, _>::get(SEASON_ID_1), None);
		assert_eq!(SeasonMetadatas::<Test, _>::get(SEASON_ID_1), None);
		assert_eq!(SeasonSchedules::<Test, _>::get(SEASON_ID_1), None);
		assert_eq!(SeasonScheduledActions::<Test, _>::iter().count(), 0);
		assert_eq!(AssetSeasonRegister::<Test, _>::iter().count(), 0);

		assert_ok!(SeasonsAlpha::update_season(
			RuntimeOrigin::signed(ALICE),
			SEASON_ID_1,
			Some(config.clone()),
			Some(metadata.clone()),
			Some(schedule.clone())
		));

		System::assert_last_event(RuntimeEvent::SeasonsAlpha(Event::UpdatedSeason {
			season_id: SEASON_ID_1,
			config: Some(config.clone()),
			metadata: Some(metadata.clone()),
			schedule: Some(schedule.clone()),
		}));

		// After updating all data for the first season some data appears
		assert_err!(CurrentSeasonStatus::<Test, _>::get(), Error::<Test, _>::NoActiveSeason);
		assert_eq!(LatestSeason::<Test, _>::get(), Some(SEASON_ID_1));
		assert_eq!(FinishedSeasons::<Test, _>::iter().count(), 0);
		assert_eq!(NextSeasonChain::<Test, _>::iter().count(), 0);
		assert_eq!(PrevSeasonChain::<Test, _>::iter().count(), 0);
		assert_eq!(Seasons::<Test, _>::get(SEASON_ID_1), Some(config.clone()));
		assert_eq!(SeasonMetadatas::<Test, _>::get(SEASON_ID_1), Some(metadata));
		assert_eq!(SeasonSchedules::<Test, _>::get(SEASON_ID_1), Some(schedule));
		assert_eq!(SeasonScheduledActions::<Test, _>::iter().count(), 3);
		assert_eq!(AssetSeasonRegister::<Test, _>::iter().count(), 0);

		// We move to the early starting block of season 1
		run_to_block(20);

		System::assert_last_event(RuntimeEvent::SeasonsAlpha(Event::SeasonEarlyStarted {
			season_id: SEASON_ID_1,
		}));

		let expected_status =
			SeasonStatus { season_id: SEASON_ID_1, early: true, active: true, early_ended: false };

		assert_eq!(CurrentSeasonStatus::<Test, _>::get(), Ok(expected_status));

		// We move to the starting block of season 1 and we see that nothing happens
		// since the season already early started
		run_to_block(25);

		System::assert_last_event(RuntimeEvent::SeasonsAlpha(Event::SeasonEarlyStarted {
			season_id: SEASON_ID_1,
		}));

		let metadata = SeasonMetadata {
			name: BoundedVec::try_from(b"Season-2".to_vec()).expect("Should create vec"),
			description: BoundedVec::try_from(b"The second season".to_vec())
				.expect("Should create vec"),
		};
		let schedule = SeasonSchedule { early_start: 28, start: 40, end: Some(60) };

		// We create a new season with its early_start overlapping season 1
		assert_ok!(SeasonsAlpha::update_season(
			RuntimeOrigin::signed(ALICE),
			SEASON_ID_2,
			Some(config.clone()),
			Some(metadata.clone()),
			Some(schedule.clone())
		));

		System::assert_last_event(RuntimeEvent::SeasonsAlpha(Event::UpdatedSeason {
			season_id: SEASON_ID_2,
			config: Some(config.clone()),
			metadata: Some(metadata.clone()),
			schedule: Some(schedule.clone()),
		}));

		let expected_status =
			SeasonStatus { season_id: SEASON_ID_1, early: true, active: true, early_ended: false };

		assert_eq!(CurrentSeasonStatus::<Test, _>::get(), Ok(expected_status));
		assert_eq!(LatestSeason::<Test, _>::get(), Some(SEASON_ID_2));
		assert_eq!(FinishedSeasons::<Test, _>::iter().count(), 0);
		assert_eq!(NextSeasonChain::<Test, _>::iter().count(), 1);
		assert_eq!(NextSeasonChain::<Test, _>::get(SEASON_ID_1), Some(SEASON_ID_2));
		assert_eq!(NextSeasonChain::<Test, _>::get(SEASON_ID_2), None);
		assert_eq!(PrevSeasonChain::<Test, _>::iter().count(), 1);
		assert_eq!(PrevSeasonChain::<Test, _>::get(SEASON_ID_1), None);
		assert_eq!(PrevSeasonChain::<Test, _>::get(SEASON_ID_2), Some(SEASON_ID_1));
		assert_eq!(Seasons::<Test, _>::get(SEASON_ID_2), Some(config.clone()));
		assert_eq!(SeasonMetadatas::<Test, _>::get(SEASON_ID_2), Some(metadata));
		assert_eq!(SeasonSchedules::<Test, _>::get(SEASON_ID_2), Some(schedule));
		assert_eq!(SeasonScheduledActions::<Test, _>::iter().count(), 4);
		assert_eq!(AssetSeasonRegister::<Test, _>::iter().count(), 0);

		// We move to the block in which season 2 should early start
		// we see that the only thing that happens is that the action for
		// season 2 early start is consumed but season 1 reamins the current active season
		run_to_block(28);

		let expected_status =
			SeasonStatus { season_id: SEASON_ID_1, early: true, active: true, early_ended: false };

		assert_eq!(CurrentSeasonStatus::<Test, _>::get(), Ok(expected_status));
		assert_eq!(SeasonScheduledActions::<Test, _>::iter().count(), 3);

		// We move to season 1 ending block
		run_to_block(30);

		System::assert_last_event(RuntimeEvent::SeasonsAlpha(Event::SeasonEnded {
			season_id: SEASON_ID_1,
		}));

		let expected_status =
			SeasonStatus { season_id: SEASON_ID_1, early: true, active: false, early_ended: false };

		assert_eq!(CurrentSeasonStatus::<Test, _>::get(), Ok(expected_status));
		assert_eq!(FinishedSeasons::<Test, _>::iter().count(), 1);
		assert_eq!(SeasonScheduledActions::<Test, _>::iter().count(), 2);

		// We now move to season 2 start block and season 2 manages to start
		// but not early as indicated in the status
		run_to_block(40);

		System::assert_last_event(RuntimeEvent::SeasonsAlpha(Event::SeasonStarted {
			season_id: SEASON_ID_2,
		}));

		let expected_status =
			SeasonStatus { season_id: SEASON_ID_2, early: false, active: true, early_ended: false };

		assert_eq!(CurrentSeasonStatus::<Test, _>::get(), Ok(expected_status));
		assert_eq!(FinishedSeasons::<Test, _>::iter().count(), 1);
		assert_eq!(SeasonScheduledActions::<Test, _>::iter().count(), 1);

		// We move to block 50, 10 block before season 2 ends and we interrupt it
		// ending the season early
		run_to_block(50);

		assert_ok!(SeasonsAlpha::interrupt_active_season(RuntimeOrigin::signed(ALICE),));

		System::assert_last_event(RuntimeEvent::SeasonsAlpha(Event::SeasonEarlyEnded {
			season_id: SEASON_ID_2,
		}));

		let expected_status =
			SeasonStatus { season_id: SEASON_ID_2, early: false, active: false, early_ended: true };

		assert_eq!(CurrentSeasonStatus::<Test, _>::get(), Ok(expected_status));
		assert_eq!(FinishedSeasons::<Test, _>::iter().count(), 2);
		assert_eq!(SeasonScheduledActions::<Test, _>::iter().count(), 0);
	});
}

mod update_season {
	use super::*;

	#[test]
	fn update_season_works() {
		ExtBuilder::default().organizer(BOB).build().execute_with(|| {
			run_to_block(10);

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
			};
			let metadata = SeasonMetadata {
				name: BoundedVec::try_from(b"Season-1".to_vec()).expect("Should create vec"),
				description: BoundedVec::try_from(b"The first season".to_vec())
					.expect("Should create vec"),
			};
			let season_early_start = 20;
			let season_start = 25;
			let season_end = 30;
			let schedule = SeasonSchedule {
				early_start: season_early_start,
				start: season_start,
				end: Some(season_end),
			};

			assert_eq!(Seasons::<Test, _>::get(SEASON_ID_1), None);
			assert_ok!(SeasonsAlpha::update_season(
				RuntimeOrigin::signed(BOB),
				SEASON_ID_1,
				Some(config.clone()),
				None,
				None
			));
			System::assert_last_event(RuntimeEvent::SeasonsAlpha(Event::UpdatedSeason {
				season_id: SEASON_ID_1,
				config: Some(config.clone()),
				metadata: None,
				schedule: None,
			}));
			assert_eq!(Seasons::<Test, _>::get(SEASON_ID_1), Some(config));

			run_to_block(12);

			assert_eq!(SeasonMetadatas::<Test, _>::get(SEASON_ID_1), None);
			assert_ok!(SeasonsAlpha::update_season(
				RuntimeOrigin::signed(BOB),
				SEASON_ID_1,
				None,
				Some(metadata.clone()),
				None
			));
			System::assert_last_event(RuntimeEvent::SeasonsAlpha(Event::UpdatedSeason {
				season_id: SEASON_ID_1,
				config: None,
				metadata: Some(metadata.clone()),
				schedule: None,
			}));
			assert_eq!(SeasonMetadatas::<Test, _>::get(SEASON_ID_1), Some(metadata));

			run_to_block(15);

			assert_eq!(SeasonSchedules::<Test, _>::get(SEASON_ID_1), None);
			assert_eq!(SeasonScheduledActions::<Test, _>::get(season_early_start), None);
			assert_eq!(SeasonScheduledActions::<Test, _>::get(season_start), None);
			assert_eq!(SeasonScheduledActions::<Test, _>::get(season_end), None);
			assert_ok!(SeasonsAlpha::update_season(
				RuntimeOrigin::signed(BOB),
				SEASON_ID_1,
				None,
				None,
				Some(schedule.clone())
			));
			System::assert_last_event(RuntimeEvent::SeasonsAlpha(Event::UpdatedSeason {
				season_id: SEASON_ID_1,
				config: None,
				metadata: None,
				schedule: Some(schedule.clone()),
			}));
			assert_eq!(SeasonSchedules::<Test, _>::get(SEASON_ID_1), Some(schedule));
			assert_eq!(
				SeasonScheduledActions::<Test, _>::get(season_early_start),
				Some(SeasonScheduledAction::EarlyStart(SEASON_ID_1))
			);
			assert_eq!(
				SeasonScheduledActions::<Test, _>::get(season_start),
				Some(SeasonScheduledAction::Start(SEASON_ID_1))
			);
			assert_eq!(
				SeasonScheduledActions::<Test, _>::get(season_end),
				Some(SeasonScheduledAction::End(SEASON_ID_1))
			);
		});
	}

	#[test]
	fn update_season_should_reject_invalid_schedules() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			run_to_block(10);

			// Season early_start is before current block
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
			};
			let schedule = SeasonSchedule { early_start: 7, start: 10, end: Some(11) };
			assert_noop!(
				SeasonsAlpha::update_season(
					RuntimeOrigin::signed(ALICE),
					SEASON_ID_1,
					Some(config.clone()),
					None,
					Some(schedule)
				),
				Error::<Test, _>::SeasonStartBeforeCurrentBlock
			);

			// Season start is before early_start
			let schedule = SeasonSchedule { early_start: 11, start: 10, end: Some(13) };
			assert_noop!(
				SeasonsAlpha::update_season(
					RuntimeOrigin::signed(ALICE),
					SEASON_ID_1,
					Some(config.clone()),
					None,
					Some(schedule)
				),
				Error::<Test, _>::SeasonStartBeforeEarlyStart
			);

			// Season end is before start
			let schedule = SeasonSchedule { early_start: 11, start: 15, end: Some(13) };
			assert_noop!(
				SeasonsAlpha::update_season(
					RuntimeOrigin::signed(ALICE),
					SEASON_ID_1,
					Some(config.clone()),
					None,
					Some(schedule)
				),
				Error::<Test, _>::SeasonEndBeforeStart
			);

			// Season early_start is before previous season start
			let schedule_1 = SeasonSchedule { early_start: 15, start: 17, end: Some(20) };
			assert_ok!(SeasonsAlpha::update_season(
				RuntimeOrigin::signed(ALICE),
				SEASON_ID_1,
				Some(config.clone()),
				None,
				Some(schedule_1)
			));
			let schedule_2 = SeasonSchedule { early_start: 13, start: 20, end: Some(25) };
			assert_noop!(
				SeasonsAlpha::update_season(
					RuntimeOrigin::signed(ALICE),
					SEASON_ID_2,
					Some(config),
					None,
					Some(schedule_2)
				),
				Error::<Test, _>::SeasonStartOverlapsPreviousSeason
			);
		});
	}

	#[test]
	fn update_season_should_reject_updating_overtaken_season() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			run_to_block(10);

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
			};
			let schedule = SeasonSchedule { early_start: 20, start: 25, end: Some(30) };

			// We set up the first season but with no schedule
			assert_ok!(SeasonsAlpha::update_season(
				RuntimeOrigin::signed(ALICE),
				SEASON_ID_1,
				Some(config.clone()),
				None,
				None
			));

			// We set up the second season with a schedule
			assert_ok!(SeasonsAlpha::update_season(
				RuntimeOrigin::signed(ALICE),
				SEASON_ID_2,
				Some(config),
				None,
				Some(schedule),
			));

			// Now we run to the starting block so the schedule for season 2 can begin
			// doing so locks away the possibility of ever starting season 1
			run_to_block(20);

			let expected_status = SeasonStatus {
				season_id: SEASON_ID_2,
				early: true,
				active: true,
				early_ended: false,
			};
			assert_eq!(CurrentSeasonStatus::<Test, _>::get(), Ok(expected_status));

			// Trying to insert a schedule for season 1 fails since 2 is already active
			let schedule = SeasonSchedule { early_start: 40, start: 50, end: Some(60) };
			assert_noop!(
				SeasonsAlpha::update_season(
					RuntimeOrigin::signed(ALICE),
					SEASON_ID_1,
					None,
					None,
					Some(schedule.clone()),
				),
				Error::<Test, _>::SeasonStartOverlapsNextSeason
			);

			// Even when season 2 is finished we still cannot insert the schedule for season 1
			run_to_block(30);

			let expected_status = SeasonStatus {
				season_id: SEASON_ID_2,
				early: true,
				active: false,
				early_ended: false,
			};
			assert_eq!(CurrentSeasonStatus::<Test, _>::get(), Ok(expected_status));

			assert_noop!(
				SeasonsAlpha::update_season(
					RuntimeOrigin::signed(ALICE),
					SEASON_ID_1,
					None,
					None,
					Some(schedule),
				),
				Error::<Test, _>::SeasonStartOverlapsNextSeason
			);

			// Trying again after the previous season block end also doesn't work
			run_to_block(40);

			let schedule = SeasonSchedule { early_start: 45, start: 50, end: Some(60) };
			assert_noop!(
				SeasonsAlpha::update_season(
					RuntimeOrigin::signed(ALICE),
					SEASON_ID_1,
					None,
					None,
					Some(schedule),
				),
				Error::<Test, _>::SeasonStartOverlapsNextSeason
			);
		});
	}

	#[test]
	fn update_season_rejects_scheduling_season_without_config_set() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			run_to_block(10);

			let schedule = SeasonSchedule { early_start: 20, start: 25, end: Some(30) };
			assert_noop!(
				SeasonsAlpha::update_season(
					RuntimeOrigin::signed(ALICE),
					SEASON_ID_1,
					None,
					None,
					Some(schedule)
				),
				Error::<Test, _>::CannotScheduleSeasonWithoutConfig
			);
		});
	}

	#[test]
	fn update_season_rejects_non_organizer() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			run_to_block(10);

			assert_noop!(
				SeasonsAlpha::update_season(
					RuntimeOrigin::signed(BOB),
					SEASON_ID_1,
					None,
					None,
					None,
				),
				DispatchError::Other(ACCOUNT_IS_NOT_ORGANIZER)
			);
		});
	}

	#[test]
	fn update_season_ignores_changing_active_or_already_finished_season() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			run_to_block(10);

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
			};
			let schedule = SeasonSchedule { early_start: 20, start: 25, end: Some(30) };

			// We set up the first season but with no schedule
			assert_ok!(SeasonsAlpha::update_season(
				RuntimeOrigin::signed(ALICE),
				SEASON_ID_1,
				Some(config.clone()),
				None,
				Some(schedule.clone())
			));

			run_to_block(20);

			System::assert_last_event(RuntimeEvent::SeasonsAlpha(Event::SeasonEarlyStarted {
				season_id: SEASON_ID_1,
			}));

			let new_config = SeasonConfigOf::<Test, _> {
				fee: SeasonFeeConfig {
					transfer_asset: 12_u64,
					buy_asset_min: 3_u64,
					buy_percent: 20,
					upgrade_asset_inventory: 7_u64,
					unlock_trade_asset: 9_u64,
					unlock_transfer_asset: 13_u64,
					state_transition_base_fee: 23_u64,
				},
			};
			let new_schedule = SeasonSchedule { early_start: 20, start: 25, end: Some(30) };

			assert_ok!(SeasonsAlpha::update_season(
				RuntimeOrigin::signed(ALICE),
				SEASON_ID_1,
				Some(new_config.clone()),
				None,
				Some(new_schedule.clone()),
			));

			// Nothing has changed
			assert_eq!(Seasons::<Test, _>::get(SEASON_ID_1), Some(config.clone()));
			assert_eq!(SeasonSchedules::<Test, _>::get(SEASON_ID_1), Some(schedule.clone()));

			run_to_block(30);

			System::assert_last_event(RuntimeEvent::SeasonsAlpha(Event::SeasonEnded {
				season_id: SEASON_ID_1,
			}));

			assert_ok!(SeasonsAlpha::update_season(
				RuntimeOrigin::signed(ALICE),
				SEASON_ID_1,
				Some(new_config),
				None,
				Some(new_schedule),
			));

			// Nothing has changed
			assert_eq!(Seasons::<Test, _>::get(SEASON_ID_1), Some(config));
			assert_eq!(SeasonSchedules::<Test, _>::get(SEASON_ID_1), Some(schedule));
		});
	}

	#[test]
	fn update_season_works_with_long_season_chains() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			run_to_block(10);

			for season_id in 0..100_u32 {
				assert_ok!(SeasonsAlpha::update_season(
					RuntimeOrigin::signed(ALICE),
					season_id,
					None,
					None,
					None,
				));
			}

			for season_id in 0..100_u32 {
				let expected_prev_id = if season_id == 0 { None } else { Some(season_id - 1) };
				let expected_next_id = if season_id == 99 { None } else { Some(season_id + 1) };
				assert_eq!(PrevSeasonChain::<Test, _>::get(season_id), expected_prev_id);
				assert_eq!(NextSeasonChain::<Test, _>::get(season_id), expected_next_id);
			}
		});
	}

	#[test]
	fn update_season_should_allow_updating_infinite_active_season() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			run_to_block(10);

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
			};
			let schedule = SeasonSchedule { early_start: 20, start: 25, end: None };

			// We set an infinite season
			assert_ok!(SeasonsAlpha::update_season(
				RuntimeOrigin::signed(ALICE),
				SEASON_ID_1,
				Some(config.clone()),
				None,
				Some(schedule.clone())
			));

			run_to_block(40);

			let expected_status = SeasonStatus {
				season_id: SEASON_ID_1,
				early: true,
				active: true,
				early_ended: false,
			};
			assert_eq!(CurrentSeasonStatus::<Test, _>::get(), Ok(expected_status));

			let schedule = SeasonSchedule { early_start: 30, start: 35, end: Some(60) };
			assert_ok!(SeasonsAlpha::update_season(
				RuntimeOrigin::signed(ALICE),
				SEASON_ID_1,
				None,
				None,
				Some(schedule.clone())
			));

			// Only the end schedule has changed
			let expected_schedule = SeasonSchedule { early_start: 20, start: 25, end: Some(60) };
			assert_eq!(SeasonSchedules::<Test, _>::get(SEASON_ID_1), Some(expected_schedule));

			run_to_block(60);

			System::assert_last_event(RuntimeEvent::SeasonsAlpha(Event::SeasonEnded {
				season_id: SEASON_ID_1,
			}));

			let expected_status = SeasonStatus {
				season_id: SEASON_ID_1,
				early: true,
				active: false,
				early_ended: false,
			};
			assert_eq!(CurrentSeasonStatus::<Test, _>::get(), Ok(expected_status));
		});
	}
}

mod interrupt_active_season {
	use super::*;

	#[test]
	fn interrupt_active_season_works() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			run_to_block(10);

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
			};
			let schedule = SeasonSchedule { early_start: 20, start: 25, end: Some(30) };

			assert_ok!(SeasonsAlpha::update_season(
				RuntimeOrigin::signed(ALICE),
				SEASON_ID_1,
				Some(config.clone()),
				None,
				Some(schedule.clone())
			));

			run_to_block(20);

			System::assert_last_event(RuntimeEvent::SeasonsAlpha(Event::SeasonEarlyStarted {
				season_id: SEASON_ID_1,
			}));

			let expected_status = SeasonStatus {
				season_id: SEASON_ID_1,
				early: true,
				active: true,
				early_ended: false,
			};
			assert_eq!(CurrentSeasonStatus::<Test, _>::get(), Ok(expected_status));
			assert_eq!(SeasonScheduledActions::<Test, _>::iter().count(), 2);

			assert_ok!(SeasonsAlpha::interrupt_active_season(RuntimeOrigin::signed(ALICE)));

			System::assert_last_event(RuntimeEvent::SeasonsAlpha(Event::SeasonEarlyEnded {
				season_id: SEASON_ID_1,
			}));
			let expected_status = SeasonStatus {
				season_id: SEASON_ID_1,
				early: true,
				active: false,
				early_ended: true,
			};
			assert_eq!(CurrentSeasonStatus::<Test, _>::get(), Ok(expected_status));
			assert_eq!(SeasonScheduledActions::<Test, _>::iter().count(), 0);
		});
	}

	#[test]
	fn interrupt_active_season_rejects_non_organizer_calls() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			run_to_block(10);

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
			};
			let schedule = SeasonSchedule { early_start: 20, start: 25, end: Some(30) };

			assert_ok!(SeasonsAlpha::update_season(
				RuntimeOrigin::signed(ALICE),
				SEASON_ID_1,
				Some(config.clone()),
				None,
				Some(schedule.clone())
			));

			run_to_block(20);

			System::assert_last_event(RuntimeEvent::SeasonsAlpha(Event::SeasonEarlyStarted {
				season_id: SEASON_ID_1,
			}));

			assert_noop!(
				SeasonsAlpha::interrupt_active_season(RuntimeOrigin::signed(BOB)),
				DispatchError::Other(ACCOUNT_IS_NOT_ORGANIZER)
			);
		});
	}
}
