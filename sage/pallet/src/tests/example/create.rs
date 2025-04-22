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
fn create_player_works() {
	let initial_balance = 100_000;
	ExtBuilder::default()
		.balances(&[(alice(), initial_balance)])
		.build()
		.execute_with(|| {
			let transition_id = CasinoAction::Create(AssetType::Player);

			assert_eq!(Sage::iter_assets_from(&alice()).count(), 0);

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

			let assets = Sage::iter_assets_from(&alice()).collect::<Vec<_>>();
			assert_eq!(assets.len(), 2);

			let (_, mut asset_1) = assets[0];
			let (_, mut asset_2) = assets[1];

			let (human, tracker) =
				if asset_1.variant.is_variant(VariantType::Player(PlayerType::Human)) {
					assert!(asset_2.variant.is_variant(VariantType::Player(PlayerType::Tracker)));

					let player_1 = asset_1.try_as_player().expect("Should be player");
					let human = player_1.try_as_human().expect("Should be human");
					let player_2 = asset_2.try_as_player().expect("Should be player");
					let tracker = player_2.try_as_tracker().expect("Should be tracker");

					(human, tracker)
				} else {
					assert!(asset_1.variant.is_variant(VariantType::Player(PlayerType::Tracker)));
					assert!(asset_2.variant.is_variant(VariantType::Player(PlayerType::Human)));

					let player_1 = asset_1.try_as_player().expect("Should be player");
					let human = player_1.try_as_human().expect("Should be human");
					let player_2 = asset_2.try_as_player().expect("Should be player");
					let tracker = player_2.try_as_tracker().expect("Should be tracker");

					(human, tracker)
				};

			assert!(human.seat_id.is_none());
			assert_eq!(tracker.slot_a_result, 0);
			assert_eq!(tracker.slot_b_result, 0);
			assert_eq!(tracker.slot_c_result, 0);
			assert_eq!(tracker.slot_d_result, 0);
			assert_eq!(tracker.last_reward, 0);
		});
}

#[test]
fn create_machine_works() {
	let initial_balance = 100_000;
	ExtBuilder::default()
		.balances(&[(alice(), initial_balance)])
		.build()
		.execute_with(|| {
			let transition_id = CasinoAction::Create(AssetType::Machine(MachineType::Bandit));

			assert_eq!(Sage::iter_assets_from(&alice()).count(), 0);

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

			let assets = Sage::iter_assets_from(&alice()).collect::<Vec<_>>();
			assert_eq!(assets.len(), 1);

			let (_, mut machine_asset) = assets[0];

			assert!(machine_asset.variant.is_variant(VariantType::Machine(MachineType::Bandit)));

			let machine = machine_asset.try_as_machine().expect("Should be MachineType::Bandit");

			assert_eq!(machine.seat_linked, 0);
			assert_eq!(machine.seat_limit, 1);
			assert_eq!(machine.value_1_factor, TokenType::T1);
			assert_eq!(machine.value_1_mul, MultiplierType::V1);
			assert_eq!(machine.value_2_factor, TokenType::T1);
			assert_eq!(machine.value_2_mul, MultiplierType::V0);
			assert_eq!(machine.value_3_factor, TokenType::T1);
			assert_eq!(machine.value_3_mul, MultiplierType::V0);

			let bandit = machine.try_as_bandit().expect("Should be Bandit");

			assert_eq!(bandit.max_spins, 4);
			assert_eq!(bandit.jackpot, 0);
		});
}
