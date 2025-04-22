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
use sp_runtime::SaturatedConversion;

#[test]
fn rent_seat_works() {
	let initial_balance = 100_000;
	ExtBuilder::default()
		.balances(&[(alice(), initial_balance)])
		.build()
		.execute_with(|| {
			create_machine_for(alice());

			assert_eq!(Sage::iter_assets_from(&alice()).count(), 1);

			let (machine_id, _) =
				get_assets_from(alice(), VariantType::Machine(MachineType::Bandit))[0];

			let multiplier = MultiplierType::V1;
			let transition_id = CasinoAction::Rent(multiplier);

			assert_ok!(Sage::state_transition(
				RuntimeOrigin::signed(alice()),
				transition_id,
				vec![machine_id],
				(),
				SOME_NATIVE_PAYMENT
			));

			assert_eq!(Sage::iter_assets_from(&alice()).count(), 2);

			let (_, mut seat_asset) = get_assets_from(alice(), VariantType::Seat)[0];

			let seat = seat_asset.try_as_seat().expect("should have seat");

			assert_eq!(seat.seat_validity_period, multiplier.as_seat_validity_period());
			assert_eq!(seat.player_fee, 1);
			assert_eq!(seat.player_grace_period, 30);
			assert_eq!(
				seat.reservation_start_block,
				0_u32.saturated_into::<BlockNumberFor<Test>>()
			);
			assert_eq!(seat.reservation_duration, 0);
			assert_eq!(seat.last_action_block, 0);
			assert_eq!(seat.player_action_count, 0);
			assert_eq!(seat.player_id, None);
			assert_eq!(seat.machine_id, Some(machine_id));
		});
}
