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
fn set_organizer_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Organizer::<Test, Instance1>::get(), None);
		assert_ok!(Sage::set_organizer(RuntimeOrigin::root(), ALICE));
		assert_eq!(Organizer::<Test, Instance1>::get(), Some(ALICE));
		System::assert_last_event(RuntimeEvent::Sage(Event::OrganizerSet { organizer: ALICE }));
	});
}

#[test]
fn set_organizer_should_reject_non_root_calls() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Sage::set_organizer(RuntimeOrigin::signed(ALICE), BOB),
			DispatchError::BadOrigin
		);
	});
}

#[test]
fn set_organizer_should_replace_existing_organizer() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Organizer::<Test, Instance1>::get(), None);
		assert_ok!(Sage::set_organizer(RuntimeOrigin::root(), DAVE));
		assert_eq!(Organizer::<Test, Instance1>::get(), Some(DAVE));
		System::assert_last_event(RuntimeEvent::Sage(Event::OrganizerSet { organizer: DAVE }));

		assert_ok!(Sage::set_organizer(RuntimeOrigin::root(), CHARLIE));
		assert_eq!(Organizer::<Test, Instance1>::get(), Some(CHARLIE));
		System::assert_last_event(RuntimeEvent::Sage(Event::OrganizerSet { organizer: CHARLIE }));
	});
}
