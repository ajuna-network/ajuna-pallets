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

mod extrinsics;
mod pallet_impls;
mod trait_impls;

use crate::{mock::*, *};
use example_transition::types::{Asset, AssetId, Level};

use frame_support::{assert_noop, assert_ok};

pub(crate) fn create_assets<I: 'static>(
	season_id: MockSeasonId,
	account: MockAccountId,
	n: u8,
) -> Vec<AssetIdOf<Test, I>>
where
	Test: Config<I>,
	AssetIdOf<Test, I>: From<[u8; 32]>,
	AssetOf<Test, I>: From<Asset>,
	SeasonIdOf<Test, I>: From<MockSeasonId>,
{
	let casted_season_id = SeasonIdOf::<Test, I>::from(season_id);
	AssetsOwnedCount::<Test, I>::mutate(account, &casted_season_id, |asset_count| {
		*asset_count = asset_count.saturating_add(n);
	});

	(0..n)
		.map(|i| {
			let asset_id = AssetId::random();
			ASSET_SEASONS.with_borrow_mut(|store| {
				store.insert(asset_id, season_id);
			});
			let asset_instance = Asset::create(asset_id, 0, 0, 0, [i; 32], 0, Level::One);

			let asset_id = AssetIdOf::<Test, I>::from(asset_id.0);
			let asset = AssetOf::<Test, I>::from(asset_instance);
			Assets::<Test, I>::insert(&asset_id, (account, asset));
			AssetOwners::<Test, I>::insert((account, &casted_season_id, &asset_id), ());
			asset_id
		})
		.collect()
}
