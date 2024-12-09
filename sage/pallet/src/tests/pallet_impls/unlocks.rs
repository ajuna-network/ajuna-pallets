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

use super::*;

mod unlock_asset_trading_for {
	use super::*;

	#[test]
	fn unlock_asset_trading_for_oneself_free_should_reject_if_feature_locked_in_season() {
		ExtBuilder::default().build().execute_with(|| {
			assert_noop!(
				Sage::unlock_asset_trading_for(DAVE, UnlockTarget::OneselfFree, SEASON_ID_0),
				Error::<Test, ()>::FeatureUnavailableInSeason
			);
		});
	}

	#[test]
	fn unlock_asset_trading_for_oneself_free_should_reject_if_criteria_not_fullfilled() {
		ExtBuilder::default().build().execute_with(|| {
			let unlock_rule = UnlockRule::from([2, 0, 3, 1, 1]);
			SeasonUnlocks::<Test, ()>::insert(
				SEASON_ID_0,
				LockableFeature::TradeAsset,
				unlock_rule,
			);

			let player_stats = PlayerSeasonStats::<Test, ()>::get(DAVE, SEASON_ID_0);
			// We don't fullfill the criteria
			assert_eq!(player_stats.minted_amount, 0);
			assert_eq!(player_stats.forged_amount, 0);
			assert_eq!(player_stats.bought_amount, 0);
			assert_eq!(player_stats.sold_amount, 0);
			assert_noop!(
				Sage::unlock_asset_trading_for(DAVE, UnlockTarget::OneselfFree, SEASON_ID_0),
				Error::<Test, ()>::UnlockCriteriaNotFulfilled
			);

			// Changed it to pass the criteria
			PlayerSeasonStats::<Test, ()>::mutate(DAVE, SEASON_ID_0, |stats| {
				stats.minted_amount = 2;
				stats.forged_amount = 3;
				stats.bought_amount = 1;
				stats.sold_amount = 1;
			});
			assert_ok!(Sage::unlock_asset_trading_for(
				DAVE,
				UnlockTarget::OneselfFree,
				SEASON_ID_0
			));
		});
	}

	#[test]
	fn unlock_asset_trading_through_payment_should_ignore_season_lock_on_feature() {
		ExtBuilder::default().balances(&[(DAVE, 1_000)]).build().execute_with(|| {
			assert_eq!(
				SeasonUnlocks::<Test, ()>::get(SEASON_ID_0, LockableFeature::TradeAsset,),
				None
			);
			assert_noop!(
				Sage::unlock_asset_trading_for(DAVE, UnlockTarget::OneselfFree, SEASON_ID_0),
				Error::<Test, ()>::FeatureUnavailableInSeason
			);

			let player_config = PlayerSeasonConfigs::<Test, ()>::get(DAVE, SEASON_ID_0);
			assert!(!player_config.locks.asset_trade);
			assert_ok!(Sage::unlock_asset_trading_for(
				DAVE,
				UnlockTarget::OneselfPaying,
				SEASON_ID_0
			));
			let player_config = PlayerSeasonConfigs::<Test, ()>::get(DAVE, SEASON_ID_0);
			assert!(player_config.locks.asset_trade);

			let player_config = PlayerSeasonConfigs::<Test, ()>::get(BOB, SEASON_ID_0);
			assert!(!player_config.locks.asset_trade);
			assert_ok!(Sage::unlock_asset_trading_for(
				DAVE,
				UnlockTarget::OtherPaying(BOB),
				SEASON_ID_0
			));
			let player_config = PlayerSeasonConfigs::<Test, ()>::get(BOB, SEASON_ID_0);
			assert!(player_config.locks.asset_trade);
		});
	}
}

mod unlock_asset_transfer_for {
	use super::*;

	#[test]
	fn unlock_asset_transfer_for_oneself_free_should_reject_if_feature_locked_in_season() {
		ExtBuilder::default().build().execute_with(|| {
			assert_noop!(
				Sage::unlock_asset_transfer_for(DAVE, UnlockTarget::OneselfFree, SEASON_ID_0),
				Error::<Test, ()>::FeatureUnavailableInSeason
			);
		});
	}

	#[test]
	fn unlock_asset_transfer_for_oneself_free_should_reject_if_criteria_not_fullfilled() {
		ExtBuilder::default().build().execute_with(|| {
			let unlock_rule = UnlockRule::from([1, 0, 3, 4, 5]);
			SeasonUnlocks::<Test, ()>::insert(
				SEASON_ID_0,
				LockableFeature::TradeAsset,
				unlock_rule,
			);

			let player_stats = PlayerSeasonStats::<Test, ()>::get(DAVE, SEASON_ID_0);
			// We don't fullfill the criteria
			assert_eq!(player_stats.minted_amount, 0);
			assert_eq!(player_stats.forged_amount, 0);
			assert_eq!(player_stats.bought_amount, 0);
			assert_eq!(player_stats.sold_amount, 0);
			assert_noop!(
				Sage::unlock_asset_trading_for(DAVE, UnlockTarget::OneselfFree, SEASON_ID_0),
				Error::<Test, ()>::UnlockCriteriaNotFulfilled
			);

			// Changed it to pass the criteria
			PlayerSeasonStats::<Test, ()>::mutate(DAVE, SEASON_ID_0, |stats| {
				stats.minted_amount = 1;
				stats.forged_amount = 3;
				stats.bought_amount = 4;
				stats.sold_amount = 5;
			});
			assert_ok!(Sage::unlock_asset_trading_for(
				DAVE,
				UnlockTarget::OneselfFree,
				SEASON_ID_0
			));
		});
	}

	#[test]
	fn unlock_asset_transfer_through_payment_should_ignore_season_lock_on_feature() {
		ExtBuilder::default().balances(&[(DAVE, 1_000)]).build().execute_with(|| {
			assert_eq!(
				SeasonUnlocks::<Test, ()>::get(SEASON_ID_0, LockableFeature::TradeAsset),
				None
			);
			assert_noop!(
				Sage::unlock_asset_transfer_for(DAVE, UnlockTarget::OneselfFree, SEASON_ID_0),
				Error::<Test, ()>::FeatureUnavailableInSeason
			);

			let player_config = PlayerSeasonConfigs::<Test, ()>::get(DAVE, SEASON_ID_0);
			assert!(!player_config.locks.asset_transfer);
			assert_ok!(Sage::unlock_asset_transfer_for(
				DAVE,
				UnlockTarget::OneselfPaying,
				SEASON_ID_0
			));
			let player_config = PlayerSeasonConfigs::<Test, ()>::get(DAVE, SEASON_ID_0);
			assert!(player_config.locks.asset_transfer);

			let player_config = PlayerSeasonConfigs::<Test, ()>::get(BOB, SEASON_ID_0);
			assert!(!player_config.locks.asset_transfer);
			assert_ok!(Sage::unlock_asset_transfer_for(
				DAVE,
				UnlockTarget::OtherPaying(BOB),
				SEASON_ID_0
			));
			let player_config = PlayerSeasonConfigs::<Test, ()>::get(BOB, SEASON_ID_0);
			assert!(player_config.locks.asset_transfer);
		});
	}
}
