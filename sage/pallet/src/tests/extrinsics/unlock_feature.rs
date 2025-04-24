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

#[test]
fn unlock_feature_works_for_trade_asset_targets() {
	let initial_balance = 100_000;
	let feature_to_unlock = LockableFeature::TradeAsset;
	ExtBuilder::default()
		.balances(&[(alice(), initial_balance), (bob(), initial_balance)])
		.build()
		.execute_with(|| {
			let season_id = <Test as Config<()>>::SeasonHandler::get_current_season_id()
				.expect("Should get season_id");
			let season_config =
				<Test as Config<()>>::SeasonHandler::get_season_config_for(&season_id)
					.expect("Should get season config");
			let unlock_rule = UnlockRule::from([1, 0, 0, 0, 0]);

			SeasonUnlocks::<Test, ()>::insert(season_id, feature_to_unlock, unlock_rule);

			// Unlocking feature for oneself through the unlock filters
			assert!(!PlayerSeasonConfigs::<Test, ()>::get(alice(), season_id).locks.asset_trade);
			// The first attempt fails since ALICE doesn't pass the filters
			assert_noop!(
				Sage::unlock_feature(
					RuntimeOrigin::signed(alice()),
					UnlockTarget::OneselfFree,
					feature_to_unlock,
					season_id,
					SOME_NATIVE_PAYMENT
				),
				Error::<Test, ()>::UnlockCriteriaNotFulfilled
			);
			// We adjust her stats so that she does
			PlayerSeasonStats::<Test, ()>::mutate(alice(), season_id, |stats| {
				stats.minted_amount = 1;
			});
			assert_ok!(Sage::unlock_feature(
				RuntimeOrigin::signed(alice()),
				UnlockTarget::OneselfFree,
				feature_to_unlock,
				season_id,
				SOME_NATIVE_PAYMENT
			));

			System::assert_last_event(RuntimeEvent::Sage(Event::FeatureUnlocked {
				feature: feature_to_unlock,
				season_id,
				account: alice(),
			}));
			assert!(PlayerSeasonConfigs::<Test, ()>::get(alice(), season_id).locks.asset_trade);

			// Unlocking feature for oneself through paying
			assert_eq!(Balances::free_balance(bob()), initial_balance);
			assert!(!PlayerSeasonConfigs::<Test, ()>::get(bob(), season_id).locks.asset_trade);
			assert_ok!(Sage::unlock_feature(
				RuntimeOrigin::signed(bob()),
				UnlockTarget::OneselfPaying,
				feature_to_unlock,
				season_id,
				SOME_NATIVE_PAYMENT
			));
			System::assert_last_event(RuntimeEvent::Sage(Event::FeatureUnlocked {
				feature: feature_to_unlock,
				season_id,
				account: bob(),
			}));
			assert_eq!(
				Balances::free_balance(bob()),
				initial_balance - season_config.fee.unlock_trade_asset
			);
			assert!(PlayerSeasonConfigs::<Test, ()>::get(bob(), season_id).locks.asset_trade);

			// Unlocking feature for another through paying
			assert!(!PlayerSeasonConfigs::<Test, ()>::get(charlie(), season_id).locks.asset_trade);
			assert_ok!(Sage::unlock_feature(
				RuntimeOrigin::signed(bob()),
				UnlockTarget::OtherPaying(charlie()),
				feature_to_unlock,
				season_id,
				SOME_NATIVE_PAYMENT,
			));
			System::assert_last_event(RuntimeEvent::Sage(Event::FeatureUnlocked {
				feature: feature_to_unlock,
				season_id,
				account: charlie(),
			}));
			assert_eq!(
				Balances::free_balance(bob()),
				initial_balance - season_config.fee.unlock_trade_asset * 2
			);
			assert!(PlayerSeasonConfigs::<Test, ()>::get(charlie(), season_id).locks.asset_trade);
		});
}

#[test]
fn unlock_feature_works_for_transfer_asset_targets() {
	let initial_balance = 100_000;
	let feature_to_unlock = LockableFeature::TransferAsset;
	ExtBuilder::default()
		.balances(&[(alice(), initial_balance), (bob(), initial_balance)])
		.build()
		.execute_with(|| {
			let season_id = <Test as Config<()>>::SeasonHandler::get_current_season_id()
				.expect("Should get season_id");
			let season_config =
				<Test as Config<()>>::SeasonHandler::get_season_config_for(&season_id)
					.expect("Should get season config");
			let unlock_rule = UnlockRule::from([0, 0, 3, 0, 0]);

			SeasonUnlocks::<Test, ()>::insert(season_id, feature_to_unlock, unlock_rule);

			// Unlocking feature for oneself through the unlock filters
			assert!(!PlayerSeasonConfigs::<Test, ()>::get(alice(), season_id).locks.asset_transfer);
			// The first attempt fails since ALICE doesn't pass the filters
			assert_noop!(
				Sage::unlock_feature(
					RuntimeOrigin::signed(alice()),
					UnlockTarget::OneselfFree,
					feature_to_unlock,
					season_id,
					SOME_NATIVE_PAYMENT
				),
				Error::<Test, ()>::UnlockCriteriaNotFulfilled
			);
			// We adjust her stats so that she does
			PlayerSeasonStats::<Test, ()>::mutate(alice(), season_id, |stats| {
				stats.forged_amount = 3;
			});
			assert_ok!(Sage::unlock_feature(
				RuntimeOrigin::signed(alice()),
				UnlockTarget::OneselfFree,
				feature_to_unlock,
				season_id,
				SOME_NATIVE_PAYMENT
			));

			System::assert_last_event(RuntimeEvent::Sage(Event::FeatureUnlocked {
				feature: feature_to_unlock,
				season_id,
				account: alice(),
			}));
			assert!(PlayerSeasonConfigs::<Test, ()>::get(alice(), season_id).locks.asset_transfer);

			// Unlocking feature for oneself through paying
			assert_eq!(Balances::free_balance(bob()), initial_balance);
			assert!(!PlayerSeasonConfigs::<Test, ()>::get(bob(), season_id).locks.asset_transfer);
			assert_ok!(Sage::unlock_feature(
				RuntimeOrigin::signed(bob()),
				UnlockTarget::OneselfPaying,
				feature_to_unlock,
				season_id,
				SOME_NATIVE_PAYMENT
			));
			System::assert_last_event(RuntimeEvent::Sage(Event::FeatureUnlocked {
				feature: feature_to_unlock,
				season_id,
				account: bob(),
			}));
			assert_eq!(
				Balances::free_balance(bob()),
				initial_balance - season_config.fee.unlock_trade_asset
			);
			assert!(PlayerSeasonConfigs::<Test, ()>::get(bob(), season_id).locks.asset_transfer);

			// Unlocking feature for another through paying
			assert!(
				!PlayerSeasonConfigs::<Test, ()>::get(charlie(), season_id).locks.asset_transfer
			);
			assert_ok!(Sage::unlock_feature(
				RuntimeOrigin::signed(bob()),
				UnlockTarget::OtherPaying(charlie()),
				feature_to_unlock,
				season_id,
				SOME_NATIVE_PAYMENT
			));
			System::assert_last_event(RuntimeEvent::Sage(Event::FeatureUnlocked {
				feature: feature_to_unlock,
				season_id,
				account: charlie(),
			}));
			assert_eq!(
				Balances::free_balance(bob()),
				initial_balance - season_config.fee.unlock_trade_asset * 2
			);
			assert!(
				PlayerSeasonConfigs::<Test, ()>::get(charlie(), season_id).locks.asset_transfer
			);
		});
}
