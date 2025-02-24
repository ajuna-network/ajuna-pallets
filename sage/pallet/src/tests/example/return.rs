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
fn reserve_works() {
	let initial_balance = 100_000;
	ExtBuilder::default()
		.balances(&[(ALICE, initial_balance)])
		.build()
		.execute_with(|| {
			create_player_and_tracker_for(ALICE);
			create_machine_for(ALICE);

			let (machine_id, _) =
				get_assets_from(ALICE, VariantType::Machine(MachineType::Bandit))[0];

			assert_ok!(Sage::state_transition(
				RuntimeOrigin::signed(ALICE),
				CasinoAction::Rent(RentDuration::Days28),
				vec![machine_id],
				(),
				SOME_NATIVE_PAYMENT
			));

			let (human_id, _) = get_assets_from(ALICE, VariantType::Player(PlayerType::Human))[0];
			assert_ok!(Sage::state_transition(
				RuntimeOrigin::signed(ALICE),
				CasinoAction::Deposit(AssetType::Player, TokenType::T1000),
				vec![human_id],
				(),
				SOME_NATIVE_PAYMENT
			));

			let (seat_id, _) = get_assets_from(ALICE, VariantType::Seat)[0];

			run_to_block(20);

			assert_ok!(Sage::state_transition(
				RuntimeOrigin::signed(ALICE),
				CasinoAction::Reserve(ReservationDuration::Mins5),
				vec![human_id, seat_id],
				(),
				SOME_NATIVE_PAYMENT
			));

			run_to_block(100);

			assert_ok!(Sage::state_transition(
				RuntimeOrigin::signed(ALICE),
				CasinoAction::Release,
				vec![human_id, seat_id],
				(),
				SOME_NATIVE_PAYMENT
			));

			run_to_block(120);

			let (_, mut machine) =
				get_assets_from(ALICE, VariantType::Machine(MachineType::Bandit))[0];
			assert_eq!(machine.try_as_machine().expect("Should be machine").seat_linked, 1);
			assert!(Sage::get_asset(&seat_id).is_ok());

			assert_ok!(Sage::state_transition(
				RuntimeOrigin::signed(ALICE),
				CasinoAction::Return,
				vec![machine_id, seat_id],
				(),
				SOME_NATIVE_PAYMENT
			));

			let (_, mut machine) =
				get_assets_from(ALICE, VariantType::Machine(MachineType::Bandit))[0];
			assert_eq!(machine.try_as_machine().expect("Should be machine").seat_linked, 0);
			assert!(Sage::get_asset(&seat_id).is_err());
		});
}
