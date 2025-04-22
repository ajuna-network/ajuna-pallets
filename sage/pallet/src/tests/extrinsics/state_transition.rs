// Ajuna Node
// Copyright (C) 2022 BlogaTech AG

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use super::*;

#[test]
fn state_transition_works() {
	let initial_balance = 100_000;
	ExtBuilder::default()
		.balances(&[(alice(), initial_balance)])
		.build()
		.execute_with(|| {
			let season_id = <Test as Config<()>>::SeasonHandler::get_current_season_id()
				.expect("Should get season_id");
			let season_config =
				<Test as Config<()>>::SeasonHandler::get_season_config_for(&season_id)
					.expect("Should get season config");
			let transition_id = CasinoAction::Create(AssetType::Player);

			assert_eq!(Balances::free_balance(alice()), initial_balance);
			assert_ok!(Sage::state_transition(
				RuntimeOrigin::signed(alice()),
				transition_id,
				vec![],
				(),
				SOME_NATIVE_PAYMENT
			));
			System::assert_last_event(RuntimeEvent::Sage(Event::TransitionExecuted {
				account: alice(),
				id: transition_id,
			}));
			let transition_fee = season_config.fee.state_transition_base_fee;
			// This assertion assumes that for the UpgradeAsset transition the fee is 2x
			// the 'state_transition_base_fee'
			// assert_eq!(Balances::free_balance(alice()), initial_balance - (transition_fee * 2));
			assert_eq!(Balances::free_balance(alice()), initial_balance - transition_fee);
		});
}

#[test]
fn state_transition_should_reject_locked_assets() {
	let initial_balance = 100_000;
	ExtBuilder::default()
		.balances(&[(alice(), initial_balance)])
		.build()
		.execute_with(|| {
			let asset_ids = create_assets::<()>(SEASON_ID_0, alice(), 1);
			let asset_id = asset_ids[0];
			let transition_id = CasinoAction::Deposit(AssetType::Player, TokenType::T10);

			assert_ok!(Sage::lock_asset(RuntimeOrigin::signed(alice()), asset_id));
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
		.balances(&[(alice(), initial_balance)])
		.build()
		.execute_with(|| {
			let asset_ids = create_assets::<()>(SEASON_ID_0, alice(), 2);
			let transition_id = CasinoAction::Deposit(AssetType::Player, TokenType::T10);
			assert_noop!(
				Sage::state_transition(
					RuntimeOrigin::signed(alice()),
					transition_id,
					asset_ids,
					(),
					SOME_NATIVE_PAYMENT
				),
				Error::<Test, ()>::AssetLength,
			);
		})
}

#[test]
fn state_transition_should_reject_too_many_input_assets() {
	let initial_balance = 100_000;
	ExtBuilder::default()
		.balances(&[(alice(), initial_balance)])
		.build()
		.execute_with(|| {
			let asset_ids =
				create_assets::<()>(SEASON_ID_0, alice(), (MAX_ASSETS_IN_TRANSITION + 1) as u8);
			let transition_id = CasinoAction::Deposit(AssetType::Player, TokenType::T10);

			assert_noop!(
				Sage::state_transition(
					RuntimeOrigin::signed(alice()),
					transition_id,
					asset_ids,
					(),
					SOME_NATIVE_PAYMENT
				),
				Error::<Test, ()>::TooManyAssetsInTransition
			);
		})
}

#[test]
fn player_fund_transition_add_funds_to_balance_works() {
	let initial_balance = 100_000;
	ExtBuilder::default()
		.balances(&[(alice(), initial_balance)])
		.build()
		.execute_with(|| {
			let asset_ids = create_assets::<()>(SEASON_ID_0, alice(), 1);
			let season_id = <Test as Config<()>>::SeasonHandler::get_current_season_id()
				.expect("Should get season_id");
			let season_config =
				<Test as Config<()>>::SeasonHandler::get_season_config_for(&season_id)
					.expect("Should get season config");
			let transition_id = CasinoAction::Deposit(AssetType::Player, TokenType::T10);

			assert_eq!(Balances::free_balance(alice()), initial_balance);
			assert_eq!(Balances::free_balance(Sage::assets_funds_pot()), 0);
			assert_ok!(Sage::state_transition(
				RuntimeOrigin::signed(alice()),
				transition_id,
				asset_ids.clone(),
				(),
				SOME_NATIVE_PAYMENT
			));
			System::assert_last_event(RuntimeEvent::Sage(Event::TransitionExecuted {
				account: alice(),
				id: transition_id,
			}));
			let transition_fee = season_config.fee.state_transition_base_fee;
			let added_balance = TokenType::T10.get_value_for(MultiplierType::V1).into();
			assert_eq!(
                Balances::free_balance(alice()),
                initial_balance - transition_fee - added_balance
			);

			assert_eq!(Balances::free_balance(Sage::assets_funds_pot()), added_balance);
			assert_eq!(
				AssetFunds::<Test, _>::get(asset_ids[0], NATIVE_PAYMENT),
				Some(added_balance)
			);
		});
}
