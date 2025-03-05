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
use example_transition::transition::{RentDuration, ReservationDuration};

#[test]
fn gamble_works() {
	let initial_balance = 1_000_000;
	ExtBuilder::default()
		.balances(&[(ALICE, initial_balance)])
		.build()
		.execute_with(|| {
			create_player_and_tracker_for(ALICE);
			create_machine_for(ALICE);

			let (machine_id, _) =
				get_assets_from(ALICE, VariantType::Machine(MachineType::Bandit))[0];
			let (human_id, _) = get_assets_from(ALICE, VariantType::Player(PlayerType::Human))[0];

			let token = TokenType::T100000;
			let token_value = token.as_value() as u64;

			assert_ok!(Sage::state_transition(
				RuntimeOrigin::signed(ALICE),
				CasinoAction::Deposit(AssetType::Player, token),
				vec![human_id],
				(),
				SOME_NATIVE_PAYMENT
			));

			assert_ok!(Sage::state_transition(
				RuntimeOrigin::signed(ALICE),
				CasinoAction::Deposit(AssetType::Machine(MachineType::Bandit), token),
				vec![machine_id],
				(),
				SOME_NATIVE_PAYMENT
			));

			assert_eq!(Sage::inspect_asset_funds(&human_id, &NATIVE_PAYMENT), token_value);
			assert_eq!(Sage::inspect_asset_funds(&machine_id, &NATIVE_PAYMENT), token_value);

			assert_ok!(Sage::state_transition(
				RuntimeOrigin::signed(ALICE),
				CasinoAction::Rent(RentDuration::Day1),
				vec![machine_id],
				(),
				SOME_NATIVE_PAYMENT
			));

			run_to_block(10);

			let (seat_id, mut seat_asset) = get_assets_from(ALICE, VariantType::Seat)[0];
			let seat = seat_asset.try_as_seat().expect("Should be seat");

			assert_eq!(seat.last_action_block, 0);
			assert_eq!(seat.player_action_count, 0);
			assert_eq!(seat.player_id, None);
			assert_eq!(seat.machine_id, Some(machine_id));

			assert_ok!(Sage::state_transition(
				RuntimeOrigin::signed(ALICE),
				CasinoAction::Reserve(ReservationDuration::Hours8),
				vec![human_id, seat_id],
				(),
				SOME_NATIVE_PAYMENT
			));

			let (tracker_id, mut tracker_asset) =
				get_assets_from(ALICE, VariantType::Player(PlayerType::Tracker))[0];
			let tracker = tracker_asset
				.try_as_player()
				.expect("Player")
				.try_as_tracker()
				.expect("Should be tracker");

			assert_eq!(tracker.slot_a_result, (0, 0));
			assert_eq!(tracker.slot_b_result, (0, 0));
			assert_eq!(tracker.slot_c_result, (0, 0));
			assert_eq!(tracker.slot_d_result, (0, 0));
			assert_eq!(tracker.last_reward, 0);

			run_to_block(20);

			assert_ok!(Sage::state_transition(
				RuntimeOrigin::signed(ALICE),
				CasinoAction::Gamble(MultiplierType::V4),
				vec![human_id, tracker_id, seat_id, machine_id],
				(),
				SOME_NATIVE_PAYMENT
			));

			let (_, mut seat_asset) = get_assets_from(ALICE, VariantType::Seat)[0];
			let seat = seat_asset.try_as_seat().expect("Should be seat");

			assert_eq!(seat.last_action_block, 10);
			assert_eq!(seat.player_action_count, 1);
			assert_eq!(seat.player_id, Some(human_id));
			assert_eq!(seat.machine_id, Some(machine_id));

			let (_, mut tracker_asset) =
				get_assets_from(ALICE, VariantType::Player(PlayerType::Tracker))[0];
			let tracker = tracker_asset
				.try_as_player()
				.expect("Player")
				.try_as_tracker()
				.expect("Should be tracker");

			assert_eq!(tracker.slot_a_result, (21856, 2));
			assert_eq!(tracker.slot_b_result, (33648, 33));
			assert_eq!(tracker.slot_c_result, (16640, 33));
			assert_eq!(tracker.slot_d_result, (5120, 20));
			assert_eq!(tracker.last_reward, 0);
		});
}
