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
use example_transition::transition::GameTransitionConfig;

#[test]
fn update_transition_config_should_work() {
	ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
		let config = GameTransitionConfig { game_fee: 10 };
		assert_ok!(Sage::update_transition_config(RuntimeOrigin::signed(ALICE), config.clone()));
		System::assert_last_event(RuntimeEvent::Sage(Event::UpdatedTransitionConfig {
			new_config: config,
		}));
	});
}

#[test]
fn update_transition_config_should_reject_non_organizer_calls() {
	ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
		assert_noop!(
			Sage::update_transition_config(
				RuntimeOrigin::signed(BOB),
				TransitionConfigOf::<Test, _>::default()
			),
			DispatchError::BadOrigin
		);
	});
}
