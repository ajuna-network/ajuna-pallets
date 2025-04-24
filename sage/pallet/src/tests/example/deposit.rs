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
use sage_testing::ExistentialDeposit;

#[test]
fn deposit_player_works() {
	let initial_balance = 100_000;
	ExtBuilder::default()
		.balances(&[(alice(), initial_balance)])
		.build()
		.execute_with(|| {
			create_player_and_tracker_for(alice());
			let transition_cost = ExistentialDeposit::get();

			let (player_id, _) =
				get_assets_from(alice(), VariantType::Player(PlayerType::Human))[0];

			let token = TokenType::T100;
			let transition_id = CasinoAction::Deposit(AssetType::Player, token);
			let token_value = token.as_value() as u128;

			assert_eq!(Balances::free_balance(alice()), initial_balance - transition_cost);
			assert_eq!(Sage::inspect_asset_funds(&player_id, &NATIVE_PAYMENT), 0);

			assert_ok!(Sage::state_transition(
				RuntimeOrigin::signed(alice()),
				transition_id,
				vec![player_id],
				(),
				SOME_NATIVE_PAYMENT
			));

			assert_eq!(
				Balances::free_balance(alice()),
				initial_balance - token_value - (transition_cost * 2)
			);
			assert_eq!(Sage::inspect_asset_funds(&player_id, &NATIVE_PAYMENT), token_value);
		});
}

#[test]
fn deposit_machine_works() {
	let initial_balance = 100_000;
	ExtBuilder::default()
		.balances(&[(alice(), initial_balance)])
		.build()
		.execute_with(|| {
			create_machine_for(alice());
			let transition_cost = ExistentialDeposit::get();

			let (machine_id, _) =
				get_assets_from(alice(), VariantType::Machine(MachineType::Bandit))[0];

			let token = TokenType::T1000;
			let transition_id =
				CasinoAction::Deposit(AssetType::Machine(MachineType::Bandit), token);
			let token_value = token.as_value() as u128;

			assert_eq!(Balances::free_balance(alice()), initial_balance - transition_cost);
			assert_eq!(Sage::inspect_asset_funds(&machine_id, &NATIVE_PAYMENT), 0);

			assert_ok!(Sage::state_transition(
				RuntimeOrigin::signed(alice()),
				transition_id,
				vec![machine_id],
				(),
				SOME_NATIVE_PAYMENT
			));

			assert_eq!(
				Balances::free_balance(alice()),
				initial_balance - token_value - (transition_cost * 2)
			);
			assert_eq!(Sage::inspect_asset_funds(&machine_id, &NATIVE_PAYMENT), token_value);
		});
}
