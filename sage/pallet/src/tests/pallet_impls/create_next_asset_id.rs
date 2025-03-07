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
fn create_next_asset_id_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(LastAssetId::<Test, ()>::get(), 0u32);

		assert_eq!(Sage::create_next_asset_id(), Some(1u32));
		assert_eq!(LastAssetId::<Test, ()>::get(), 1u32);

		assert_eq!(Sage::create_next_asset_id(), Some(2u32));
	});
}

#[test]
fn create_next_asset_id_returns_none_upon_overflow() {
	ExtBuilder::default().build().execute_with(|| {
		LastAssetId::<Test, ()>::put(u32::MAX);
		assert_eq!(Sage::create_next_asset_id(), None);
	});
}
