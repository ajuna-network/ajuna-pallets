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

use crate::{mock::*, tests::ALICE, *};
use frame_support::{assert_noop, assert_ok};

mod season_manager {
	use super::*;

	#[test]
	fn get_season_id_for_works() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {});
	}

	#[test]
	fn get_current_season_id_works_if_active_season() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {});
	}

	#[test]
	fn get_current_season_id_rejects_if_no_active_season() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {});
	}

	#[test]
	fn is_valid_season_works() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {});
	}

	#[test]
	fn get_season_config_for_works() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {});
	}

	#[test]
	fn get_season_config_for_rejects_for_invalid_season() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {});
	}

	#[test]
	fn register_asset_in_works() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {});
	}

	#[test]
	fn register_asset_in_rejects_registering_asset_to_invalid_season() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {});
	}
}
