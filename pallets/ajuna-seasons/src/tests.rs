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

use crate::{mock::*, Error, *};
use ajuna_primitives::season_manager::SeasonFeeConfig;
use frame_support::{assert_err, assert_noop, assert_ok};

pub(crate) const SEASON_ID_1: MockSeasonId = 1;
pub(crate) const SEASON_ID_2: MockSeasonId = 2;

pub(crate) const ALICE: MockAccountId = MockAccountId::new([1; 32]);
pub(crate) const BOB: MockAccountId = MockAccountId::new([2; 32]);
pub(crate) const CHARLIE: MockAccountId = MockAccountId::new([3; 32]);
pub(crate) const DAVE: MockAccountId = MockAccountId::new([4; 32]);
pub(crate) const EDWARD: MockAccountId = MockAccountId::new([5; 32]);

#[test]
fn test_seasons_full_workflow() {
	ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
		run_to_block(10);

		let config = SeasonConfigOf::<Test, Instance1> {
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
		let metadata = SeasonMetadata {
			name: BoundedVec::try_from(b"Season-1".to_vec()).expect("Should create vec"),
			description: BoundedVec::try_from(b"The first season".to_vec())
				.expect("Should create vec"),
		};
		let schedule = SeasonSchedule { early_start: 20, start: 25, end: 30 };

		// Initially there is not data in storage
		assert_err!(
			CurrentSeasonStatus::<Test, Instance1>::get(),
			Error::<Test, Instance1>::NoActiveSeason
		);
		assert_eq!(LatestSeason::<Test, Instance1>::get(), None);
		assert_eq!(FinishedSeasons::<Test, Instance1>::iter().count(), 0);
		assert_eq!(NextSeasonChain::<Test, Instance1>::iter().count(), 0);
		assert_eq!(PrevSeasonChain::<Test, Instance1>::iter().count(), 0);
		assert_eq!(Seasons::<Test, Instance1>::get(SEASON_ID_1), None);
		assert_eq!(SeasonMetadatas::<Test, Instance1>::get(SEASON_ID_1), None);
		assert_eq!(SeasonSchedules::<Test, Instance1>::get(SEASON_ID_1), None);
		assert_eq!(SeasonScheduledActions::<Test, Instance1>::iter().count(), 0);
		assert_eq!(AssetSeasonRegister::<Test, Instance1>::iter().count(), 0);

		assert_ok!(SeasonsAlpha::update_season(
			RuntimeOrigin::signed(ALICE),
			SEASON_ID_1,
			Some(config.clone()),
			Some(metadata.clone()),
			Some(schedule.clone())
		));

		// After updating all data for the first season some data appears
		assert_err!(
			CurrentSeasonStatus::<Test, Instance1>::get(),
			Error::<Test, Instance1>::NoActiveSeason
		);
		assert_eq!(LatestSeason::<Test, Instance1>::get(), Some(SEASON_ID_1));
		assert_eq!(FinishedSeasons::<Test, Instance1>::iter().count(), 0);
		assert_eq!(NextSeasonChain::<Test, Instance1>::iter().count(), 0);
		assert_eq!(PrevSeasonChain::<Test, Instance1>::iter().count(), 0);
		assert_eq!(Seasons::<Test, Instance1>::get(SEASON_ID_1), Some(config.clone()));
		assert_eq!(SeasonMetadatas::<Test, Instance1>::get(SEASON_ID_1), Some(metadata));
		assert_eq!(SeasonSchedules::<Test, Instance1>::get(SEASON_ID_1), Some(schedule));
		assert_eq!(SeasonScheduledActions::<Test, Instance1>::iter().count(), 3);
		assert_eq!(AssetSeasonRegister::<Test, Instance1>::iter().count(), 0);

		// We move to the early starting block of season 1
		run_to_block(20);

		System::assert_last_event(RuntimeEvent::SeasonsAlpha(Event::SeasonEarlyStarted {
			season_id: SEASON_ID_1,
		}));

		let expected_status =
			SeasonStatus { season_id: SEASON_ID_1, early: true, active: true, early_ended: false };

		assert_eq!(CurrentSeasonStatus::<Test, Instance1>::get(), Ok(expected_status));

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
		let schedule = SeasonSchedule { early_start: 28, start: 40, end: 60 };

		// We create a new season with its early_start overlapping season 1
		assert_ok!(SeasonsAlpha::update_season(
			RuntimeOrigin::signed(ALICE),
			SEASON_ID_2,
			Some(config.clone()),
			Some(metadata.clone()),
			Some(schedule.clone())
		));

		let expected_status =
			SeasonStatus { season_id: SEASON_ID_1, early: true, active: true, early_ended: false };

		assert_eq!(CurrentSeasonStatus::<Test, Instance1>::get(), Ok(expected_status));
		assert_eq!(LatestSeason::<Test, Instance1>::get(), Some(SEASON_ID_2));
		assert_eq!(FinishedSeasons::<Test, Instance1>::iter().count(), 0);
		assert_eq!(NextSeasonChain::<Test, Instance1>::iter().count(), 1);
		assert_eq!(NextSeasonChain::<Test, Instance1>::get(SEASON_ID_1), Some(SEASON_ID_2));
		assert_eq!(NextSeasonChain::<Test, Instance1>::get(SEASON_ID_2), None);
		assert_eq!(PrevSeasonChain::<Test, Instance1>::iter().count(), 1);
		assert_eq!(PrevSeasonChain::<Test, Instance1>::get(SEASON_ID_1), None);
		assert_eq!(PrevSeasonChain::<Test, Instance1>::get(SEASON_ID_2), Some(SEASON_ID_1));
		assert_eq!(Seasons::<Test, Instance1>::get(SEASON_ID_2), Some(config.clone()));
		assert_eq!(SeasonMetadatas::<Test, Instance1>::get(SEASON_ID_2), Some(metadata));
		assert_eq!(SeasonSchedules::<Test, Instance1>::get(SEASON_ID_2), Some(schedule));
		assert_eq!(SeasonScheduledActions::<Test, Instance1>::iter().count(), 4);
		assert_eq!(AssetSeasonRegister::<Test, Instance1>::iter().count(), 0);

		// We move to the block in which season 2 should early start
		// we see that the only thing that happens is that the action for
		// season 2 early start is consumed but season 1 reamins the current active season
		run_to_block(28);

		let expected_status =
			SeasonStatus { season_id: SEASON_ID_1, early: true, active: true, early_ended: false };

		assert_eq!(CurrentSeasonStatus::<Test, Instance1>::get(), Ok(expected_status));
		assert_eq!(SeasonScheduledActions::<Test, Instance1>::iter().count(), 3);

		// We move to season 1 ending block
		run_to_block(30);

		System::assert_last_event(RuntimeEvent::SeasonsAlpha(Event::SeasonEnded {
			season_id: SEASON_ID_1,
		}));

		let expected_status =
			SeasonStatus { season_id: SEASON_ID_1, early: true, active: false, early_ended: false };

		assert_eq!(CurrentSeasonStatus::<Test, Instance1>::get(), Ok(expected_status));
		assert_eq!(FinishedSeasons::<Test, Instance1>::iter().count(), 1);
		assert_eq!(SeasonScheduledActions::<Test, Instance1>::iter().count(), 2);

		// We now move to season 2 start block and season 2 manages to start
		// but not early as indicated in the status
		run_to_block(40);

		System::assert_last_event(RuntimeEvent::SeasonsAlpha(Event::SeasonStarted {
			season_id: SEASON_ID_2,
		}));

		let expected_status =
			SeasonStatus { season_id: SEASON_ID_2, early: false, active: true, early_ended: false };

		assert_eq!(CurrentSeasonStatus::<Test, Instance1>::get(), Ok(expected_status));
		assert_eq!(FinishedSeasons::<Test, Instance1>::iter().count(), 1);
		assert_eq!(SeasonScheduledActions::<Test, Instance1>::iter().count(), 1);

		// We move to block 50, 10 block before season 2 ends and we interrupt it
		// ending the season early
		run_to_block(50);

		assert_ok!(SeasonsAlpha::interrupt_active_season(RuntimeOrigin::signed(ALICE),));

		System::assert_last_event(RuntimeEvent::SeasonsAlpha(Event::SeasonEarlyEnded {
			season_id: SEASON_ID_2,
		}));

		let expected_status =
			SeasonStatus { season_id: SEASON_ID_2, early: false, active: false, early_ended: true };

		assert_eq!(CurrentSeasonStatus::<Test, Instance1>::get(), Ok(expected_status));
		assert_eq!(FinishedSeasons::<Test, Instance1>::iter().count(), 2);
		assert_eq!(SeasonScheduledActions::<Test, Instance1>::iter().count(), 0);
	});
}

mod update_season {
	use super::*;
}

mod interrupt_active_season {
	use super::*;
}
