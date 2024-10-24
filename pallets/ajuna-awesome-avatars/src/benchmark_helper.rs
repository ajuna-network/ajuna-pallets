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

//! Useful helper methods that can be used in benchmarks.

use crate::{types::*, Pallet as AAvatars, *};
use frame_system::{pallet_prelude::BlockNumberFor, RawOrigin};
use sp_runtime::traits::{UniqueSaturatedInto, Zero};
use sp_std::vec;

pub fn create_seasons<T: Config>(n: usize) -> Result<(), &'static str> {
	CurrentSeasonStatus::<T>::put(SeasonStatus {
		season_id: 0,
		early: false,
		active: true,
		early_ended: false,
		max_tier_avatars: 0,
	});
	for i in 0..n {
		CurrentSeasonStatus::<T>::mutate(|status| status.season_id = i as SeasonId + 1);
		Seasons::<T>::insert(
			(i + 1) as SeasonId,
			Season {
				max_tier_forges: u32::MAX,
				max_variations: 15,
				max_components: 16,
				min_sacrifices: 1,
				max_sacrifices: 4,
				tiers: vec![
					RarityTier::Common,
					RarityTier::Uncommon,
					RarityTier::Rare,
					RarityTier::Epic,
					RarityTier::Legendary,
					RarityTier::Mythical,
				]
				.try_into()
				.unwrap(),
				single_mint_probs: vec![70, 20, 5, 4, 1].try_into().unwrap(),
				batch_mint_probs: vec![40, 30, 15, 10, 5].try_into().unwrap(),
				base_prob: 0,
				per_period: BlockNumberFor::<T>::from(10_u32),
				periods: 12,
				fee: Fee {
					mint: MintFees {
						one: 550_000_000_000_u64.unique_saturated_into(), // 0.55 BAJU
						three: 500_000_000_000_u64.unique_saturated_into(), // 0.5 BAJU
						six: 450_000_000_000_u64.unique_saturated_into(), // 0.45 BAJU
					},
					transfer_avatar: 1_000_000_000_000_u64.unique_saturated_into(), // 1 BAJU
					buy_minimum: 1_000_000_000_u64.unique_saturated_into(),
					buy_percent: 1,
					upgrade_storage: 1_000_000_000_000_u64.unique_saturated_into(), // 1 BAJU
					prepare_avatar: 5_000_000_000_000_u64.unique_saturated_into(),  // 5 BAJU
					set_price_unlock: 10_000_000_000_000_u64.unique_saturated_into(), // 10 BAJU,
					avatar_transfer_unlock: 10_000_000_000_000_u64.unique_saturated_into(), // 10 BAJU,
				},
				mint_logic: LogicGeneration::First,
				forge_logic: LogicGeneration::First,
			},
		);
		SeasonMetas::<T>::insert(
			(i + 1) as SeasonId,
			SeasonMeta {
				name: [u8::MAX; 100].to_vec().try_into().unwrap(),
				description: [u8::MAX; 1_000].to_vec().try_into().unwrap(),
			},
		);
		SeasonSchedules::<T>::insert(
			(i + 1) as SeasonId,
			SeasonSchedule {
				early_start: BlockNumberFor::<T>::from((i * 10 + 1) as u32),
				start: BlockNumberFor::<T>::from((i * 10 + 5) as u32),
				end: BlockNumberFor::<T>::from((i * 10 + 10) as u32),
			},
		);
		SeasonTradeFilters::<T>::insert((i + 1) as SeasonId, TradeFilters::default());
	}
	frame_system::Pallet::<T>::set_block_number(
		SeasonSchedules::<T>::get(CurrentSeasonStatus::<T>::get().season_id)
			.unwrap()
			.start,
	);
	Ok(())
}

pub fn create_avatars<T: Config>(player: T::AccountId, n: u32) -> Result<(), &'static str> {
	create_seasons::<T>(3)?;

	PlayerConfigs::<T>::mutate(&player, |config| {
		config.free_mints = n as MintCount;
	});

	GlobalConfigs::<T>::mutate(|config| {
		config.mint.open = true;
		config.mint.cooldown = Zero::zero();
		config.forge.open = true;
		config.avatar_transfer.open = true;
		config.trade.open = true;
		config.nft_transfer.open = true;
	});

	let season_id = CurrentSeasonStatus::<T>::get().season_id;
	PlayerSeasonConfigs::<T>::mutate(&player, season_id, |config| {
		config.stats.mint.last = Zero::zero();
		config.storage_tier = StorageTier::Max;
		config.locks = Locks::all_unlocked();
	});

	for _ in 0..n {
		AAvatars::<T>::mint(
			RawOrigin::Signed(player.clone()).into(),
			MintOption {
				payment: MintPayment::Free,
				pack_size: MintPackSize::One,
				pack_type: PackType::Material,
			},
		)?;
	}
	Ok(())
}
