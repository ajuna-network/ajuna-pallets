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
fn kick_works() {
	let initial_balance = 100_000;
	ExtBuilder::default()
		.balances(&[(alice(), initial_balance), (bob(), initial_balance)])
		.build()
		.execute_with(|| {
			create_player_and_tracker_for(alice());
			create_machine_for(alice());

			let (machine_id, _) =
				get_assets_from(alice(), VariantType::Machine(MachineType::Bandit))[0];
			let multiplier = MultiplierType::V1;

			assert_ok!(Sage::state_transition(
				RuntimeOrigin::signed(alice()),
				CasinoAction::Rent(multiplier),
				vec![machine_id],
				(),
				SOME_NATIVE_PAYMENT
			));

			let (alice_human_id, _) =
				get_assets_from(alice(), VariantType::Player(PlayerType::Human))[0];
			assert_ok!(Sage::state_transition(
				RuntimeOrigin::signed(alice()),
				CasinoAction::Deposit(AssetType::Player, TokenType::T1000),
				vec![alice_human_id],
				(),
				SOME_NATIVE_PAYMENT
			));

			let (seat_id, _) = get_assets_from(alice(), VariantType::Seat)[0];

			run_to_block(20);

			assert_ok!(Sage::state_transition(
				RuntimeOrigin::signed(alice()),
				CasinoAction::Reserve(multiplier),
				vec![alice_human_id, seat_id],
				(),
				SOME_NATIVE_PAYMENT
			));

			assert_eq!(Sage::inspect_asset_funds(&seat_id, &NATIVE_PAYMENT), 1);

			let (_, mut seat_asset) = get_assets_from(alice(), VariantType::Seat)[0];
			let (_, mut human_asset) =
				get_assets_from(alice(), VariantType::Player(PlayerType::Human))[0];

			let seat = seat_asset.try_as_seat().expect("should have seat");
			assert_eq!(seat.player_id, Some(alice_human_id));
			assert_eq!(seat.reservation_start_block, 20);
			assert_eq!(seat.reservation_duration, multiplier.as_reservation_duration());
			assert_eq!(seat.last_action_block, 0);
			assert_eq!(seat.player_action_count, 0);

			let human = human_asset
				.try_as_player()
				.expect("should have player")
				.try_as_human()
				.expect("should have human");
			assert_eq!(human.seat_id, Some(seat_id));

			run_to_block(30);

			create_player_and_tracker_for(bob());

			let (bob_human_id, _) =
				get_assets_from(bob(), VariantType::Player(PlayerType::Human))[0];

			assert_ok!(Sage::state_transition(
				RuntimeOrigin::signed(bob()),
				CasinoAction::Kick,
				vec![bob_human_id, alice_human_id, seat_id],
				(),
				SOME_NATIVE_PAYMENT
			));

			assert_eq!(Sage::inspect_asset_funds(&seat_id, &NATIVE_PAYMENT), 0);
			assert_eq!(Sage::inspect_asset_funds(&bob_human_id, &NATIVE_PAYMENT), 1);

			let (_, mut seat_asset) = get_assets_from(alice(), VariantType::Seat)[0];
			let (_, mut human_asset) =
				get_assets_from(alice(), VariantType::Player(PlayerType::Human))[0];

			let seat = seat_asset.try_as_seat().expect("should have seat");
			assert_eq!(seat.player_id, None);
			assert_eq!(seat.reservation_start_block, 0);
			assert_eq!(seat.reservation_duration, 0);
			assert_eq!(seat.last_action_block, 0);
			assert_eq!(seat.player_action_count, 0);

			let human = human_asset
				.try_as_player()
				.expect("should have player")
				.try_as_human()
				.expect("should have human");
			assert_eq!(human.seat_id, None);
		});
}
