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

use example_transition::asset::{
	hero_jam::{AssetSubType, AssetType, HeroJamAsset, StateType},
	Asset,
	AssetVariant::HeroJam,
};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::{testing::H256, SaturatedConversion};

pub(crate) fn create_assets<I: 'static>(
	season_id: MockSeasonId,
	account: MockAccountId,
	n: u8,
) -> Vec<AssetIdOf<Test, I>>
where
	Test: Config<I>,
	AssetIdOf<Test, I>: From<u64>,
	AssetOf<Test, I>: From<MockAsset>,
	SeasonIdOf<Test, I>: From<MockSeasonId>,
{
	let casted_season_id = SeasonIdOf::<Test, I>::from(season_id);
	AssetsOwnedCount::<Test, I>::mutate(account, &casted_season_id, |asset_count| {
		*asset_count = asset_count.saturating_add(n);
	});

	(0..n)
		.map(|_| {
			let base = H256::random();
			let asset_id = base.to_low_u64_be();
			ASSET_SEASONS.with_borrow_mut(|store| {
				store.insert(asset_id, season_id);
			});
			let asset_instance = Asset {
				asset_variant: HeroJam(HeroJamAsset {
					id: asset_id,
					asset_type: AssetType::Hero,
					asset_subtype: AssetSubType::None,
					energy: 0,
					fatigue: 0,
					state_type: StateType::None,
					state_sub_type: 0,
					state_sub_value: 0,
					state_change_block_number: 0_u32.saturated_into(),
					balance: 10,
				}),
			};

			let asset_id = AssetIdOf::<Test, I>::from(asset_id);
			let asset = AssetOf::<Test, I>::from(asset_instance);
			Assets::<Test, I>::insert(&asset_id, (account, asset));
			AssetOwners::<Test, I>::insert((account, &casted_season_id, &asset_id), ());
			asset_id
		})
		.collect()
}
