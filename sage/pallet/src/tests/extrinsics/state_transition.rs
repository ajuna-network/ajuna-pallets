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

use example_transition::transition::{
	hero_jam::{ActionTime, HeroAction, SleepType},
	TransitionIdentifier,
};

#[test]
fn state_transition_works() {
	let initial_balance = 100_000;
	ExtBuilder::default()
		.balances(&[(ALICE, initial_balance)])
		.build()
		.execute_with(|| {
			let season_id = <Test as Config<()>>::SeasonHandler::get_current_season_id()
				.expect("Should get season_id");
			let season_config =
				<Test as Config<()>>::SeasonHandler::get_season_config_for(&season_id)
					.expect("Should get season config");
			let transition_id = TransitionIdentifier::HeroJam(HeroAction::Create);

			assert_eq!(Balances::free_balance(ALICE), initial_balance);
			assert_ok!(Sage::state_transition(
				RuntimeOrigin::signed(ALICE),
				transition_id,
				vec![],
				(),
				SOME_NATIVE_PAYMENT
			));
			System::assert_last_event(RuntimeEvent::Sage(Event::TransitionExecuted {
				account: ALICE,
				id: transition_id,
			}));
			let transition_fee = season_config.fee.state_transition_base_fee;
			// This assertion assumes that for the UpgradeAsset transition the fee is 2x
			// the 'state_transition_base_fee'
			// assert_eq!(Balances::free_balance(ALICE), initial_balance - (transition_fee * 2));
			assert_eq!(Balances::free_balance(ALICE), initial_balance - transition_fee);
		});
}

#[test]
fn state_transition_should_reject_non_owned_assets() {
	let initial_balance = 100_000;
	ExtBuilder::default()
		.balances(&[(ALICE, initial_balance)])
		.build()
		.execute_with(|| {
			let asset_ids = create_assets::<()>(SEASON_ID_0, BOB, 1);
			let transition_id = TransitionIdentifier::HeroJam(HeroAction::Sleep(
				SleepType::Normal,
				ActionTime::Short,
			));

			assert_noop!(
				Sage::state_transition(
					RuntimeOrigin::signed(ALICE),
					transition_id,
					asset_ids,
					(),
					SOME_NATIVE_PAYMENT
				),
				Error::<Test, ()>::AssetNotOwned
			);
		})
}

#[test]
fn state_transition_should_reject_locked_assets() {
	let initial_balance = 100_000;
	ExtBuilder::default()
		.balances(&[(ALICE, initial_balance)])
		.build()
		.execute_with(|| {
			let asset_ids = create_assets::<()>(SEASON_ID_0, ALICE, 1);
			let asset_id = asset_ids[0];
			let transition_id = TransitionIdentifier::HeroJam(HeroAction::Sleep(
				SleepType::Normal,
				ActionTime::Short,
			));

			assert_ok!(Sage::lock_asset(RuntimeOrigin::signed(ALICE), asset_id));
			// This call should not be possible in the real world, but we simulate it to demonstrate
			// that you cannot bypass asset locking
			assert_noop!(
				Sage::state_transition(
					RuntimeOrigin::signed(Sage::technical_account_id()),
					transition_id,
					asset_ids,
					(),
					SOME_NATIVE_PAYMENT
				),
				Error::<Test, ()>::AssetLocked
			);
		})
}

#[test]
fn state_transition_should_reject_rule_verification_failure() {
	let initial_balance = 100_000;
	ExtBuilder::default()
		.balances(&[(ALICE, initial_balance)])
		.build()
		.execute_with(|| {
			// This test assumes that the rule for 'UpgradeAsset' in
			// sage/example-transition/src/generic.rs
			// requires the input assets to be of length 1
			let asset_ids = create_assets::<()>(SEASON_ID_0, ALICE, 2);
			let transition_id = TransitionIdentifier::HeroJam(HeroAction::Sleep(
				SleepType::Normal,
				ActionTime::Short,
			));

			assert_noop!(
				Sage::state_transition(
					RuntimeOrigin::signed(ALICE),
					transition_id,
					asset_ids,
					(),
					SOME_NATIVE_PAYMENT
				),
				Error::<Test, ()>::TransitionRuleNotSatisfied,
			);
		})
}

#[test]
fn state_transition_should_reject_too_many_input_assets() {
	let initial_balance = 100_000;
	ExtBuilder::default()
		.balances(&[(ALICE, initial_balance)])
		.build()
		.execute_with(|| {
			let asset_ids =
				create_assets::<()>(SEASON_ID_0, ALICE, (MAX_ASSETS_IN_TRANSITION + 1) as u8);
			let transition_id = TransitionIdentifier::HeroJam(HeroAction::Sleep(
				SleepType::Normal,
				ActionTime::Short,
			));

			assert_noop!(
				Sage::state_transition(
					RuntimeOrigin::signed(ALICE),
					transition_id,
					asset_ids,
					(),
					SOME_NATIVE_PAYMENT
				),
				Error::<Test, ()>::TooManyAssetsInTransition
			);
		})
}
