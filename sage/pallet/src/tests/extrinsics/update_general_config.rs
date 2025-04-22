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
fn update_general_config_should_work() {
	ExtBuilder::default().organizer(alice()).build().execute_with(|| {
		let config = GeneralConfig::default();
		assert_ok!(Sage::update_general_config(RuntimeOrigin::signed(alice()), config.clone()));
		System::assert_last_event(RuntimeEvent::Sage(Event::UpdatedGeneralConfig {
			new_config: config,
		}));
	});
}

#[test]
fn update_general_config_should_reject_non_organizer_calls() {
	ExtBuilder::default().organizer(alice()).build().execute_with(|| {
		assert_noop!(
			Sage::update_general_config(RuntimeOrigin::signed(bob()), GeneralConfig::default()),
			DispatchError::BadOrigin
		);
	});
}
