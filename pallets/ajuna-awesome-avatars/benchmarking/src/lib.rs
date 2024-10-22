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

#![cfg(feature = "runtime-benchmarks")]
#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

mod mock;

use frame_benchmarking::benchmarks;
use frame_support::traits::{Currency, Get};
use frame_system::{pallet_prelude::BlockNumberFor, RawOrigin};
use pallet_ajuna_awesome_avatars::{
	benchmark_helper::{create_avatars, create_seasons},
	types::*,
	Config as AvatarsConfig, Pallet as AAvatars, *,
};
use sp_runtime::traits::{Saturating, UniqueSaturatedFrom, UniqueSaturatedInto};
use sp_std::vec;

pub struct Pallet<T: Config>(pallet_ajuna_awesome_avatars::Pallet<T>);
pub trait Config: AvatarsConfig + pallet_balances::Config {}

type AccountIdFor<T> = <T as frame_system::Config>::AccountId;
type BalanceOf<T> = <CurrencyOf<T> as Currency<AccountIdFor<T>>>::Balance;
type CurrencyOf<T> = <T as AvatarsConfig>::Currency;

fn account<T: Config>(name: &'static str) -> T::AccountId {
	let index = 0;
	let seed = 0;
	frame_benchmarking::account(name, index, seed)
}

fn assert_last_event<T: Config>(avatars_event: Event<T>) {
	let event = <T as AvatarsConfig>::RuntimeEvent::from(avatars_event);
	frame_system::Pallet::<T>::assert_last_event(event.into());
}

benchmarks! {
	mint_free {
		let name = "player";
		let caller = account::<T>(name);
		let n in 0 .. (MaxAvatarsPerPlayer::get() - 6);
		create_avatars::<T>(caller.clone(), n)?;

		PlayerConfigs::<T>::mutate(&caller, |account| account.free_mints = MintCount::MAX);

		let mint_option = MintOption { payment: MintPayment::Free, pack_size: MintPackSize::Six,
			pack_type: PackType::Material, };
	}: mint(RawOrigin::Signed(caller.clone()), mint_option)
	verify {
		let n = n as usize;
		let season_id = CurrentSeasonStatus::<T>::get().season_id;
		let avatar_ids = Owners::<T>::get(caller, season_id)[n..(n + 6)].to_vec();
		assert_last_event::<T>(Event::AvatarsMinted { avatar_ids })
	}

	mint_normal {
		let name = "player";
		let caller = account::<T>(name);

		let n in 0 .. (MaxAvatarsPerPlayer::get() - 6);
		create_avatars::<T>(caller.clone(), n)?;

		let season = Seasons::<T>::get(CurrentSeasonStatus::<T>::get().season_id).unwrap();
		let mint_fee = season.fee.mint.fee_for(&MintPackSize::Six);
		CurrencyOf::<T>::make_free_balance_be(&caller, mint_fee);

		let mint_option = MintOption { payment: MintPayment::Normal, pack_size: MintPackSize::Six,
			pack_type: PackType::Material };
	}: mint(RawOrigin::Signed(caller.clone()), mint_option)
	verify {
		let n = n as usize;
		let season_id = CurrentSeasonStatus::<T>::get().season_id;
		let avatar_ids = Owners::<T>::get(caller, season_id)[n..(n + 6)].to_vec();
		assert_last_event::<T>(Event::AvatarsMinted { avatar_ids })
	}

	forge {
		let name = "player";
		let player = account::<T>(name);
		let n in 5 .. (MaxAvatarsPerPlayer::get() - 10);
		create_avatars::<T>(player.clone(), n)?;

		let season_id = CurrentSeasonStatus::<T>::get().season_id;
		let avatar_ids = Owners::<T>::get(&player, season_id);
		let avatar_id = avatar_ids[0];
		let (_owner, original_avatar) = Avatars::<T>::get(avatar_id).unwrap();
	}: _(RawOrigin::Signed(player), avatar_id, avatar_ids[1..5].to_vec())
	verify {
		let (_owner, upgraded_avatar) = Avatars::<T>::get(avatar_id).unwrap();
		let original_tiers = original_avatar.dna.into_iter().map(|x| x >> 4);
		let upgraded_tiers = upgraded_avatar.dna.into_iter().map(|x| x >> 4);
		let upgraded_components = original_tiers.zip(upgraded_tiers).fold(
			0, |mut count, (lhs, rhs)| {
				if lhs != rhs {
					count+=1;
				}
				count
			}
		);
		assert_last_event::<T>(Event::AvatarsForged { avatar_ids: vec![(avatar_id, upgraded_components)] })
	}

	transfer_avatar_normal {
		let from = account::<T>("from");
		let to = account::<T>("to");
		let n in 1 .. MaxAvatarsPerPlayer::get();
		create_avatars::<T>(from.clone(), MaxAvatarsPerPlayer::get())?;
		create_avatars::<T>(to.clone(), MaxAvatarsPerPlayer::get() - n)?;
		let season_id = CurrentSeasonStatus::<T>::get().season_id;
		let avatar_id = Owners::<T>::get(&from, season_id)[n as usize - 1];

		let Season { fee, .. } = Seasons::<T>::get(season_id).unwrap();
		CurrencyOf::<T>::make_free_balance_be(&from, fee.transfer_avatar);
	}: transfer_avatar(RawOrigin::Signed(from.clone()), to.clone(), avatar_id)
	verify {
		assert_last_event::<T>(Event::AvatarTransferred { from, to, avatar_id })
	}

	transfer_avatar_organizer {
		let organizer = account::<T>("organizer");
		let to = account::<T>("to");
		let n in 1 .. MaxAvatarsPerPlayer::get();
		create_avatars::<T>(organizer.clone(), MaxAvatarsPerPlayer::get())?;
		create_avatars::<T>(to.clone(), MaxAvatarsPerPlayer::get() - n)?;
		let season_id = CurrentSeasonStatus::<T>::get().season_id;
		let avatar_id = Owners::<T>::get(&organizer, season_id)[n as usize - 1];

		let Season { fee, .. } = Seasons::<T>::get(season_id).unwrap();
		CurrencyOf::<T>::make_free_balance_be(&organizer, fee.transfer_avatar);
	}: transfer_avatar(RawOrigin::Signed(organizer.clone()), to.clone(), avatar_id)
	verify {
		assert_last_event::<T>(Event::AvatarTransferred { from: organizer, to, avatar_id })
	}

	transfer_free_mints {
		create_seasons::<T>(1)?;
		let from = account::<T>("from");
		let to = account::<T>("to");
		let GlobalConfig { freemint_transfer, .. } = GlobalConfigs::<T>::get();
		let free_mint_transfer_fee = freemint_transfer.free_mint_transfer_fee;
		let how_many = MintCount::MAX - free_mint_transfer_fee as MintCount;
		let season_id = CurrentSeasonStatus::<T>::get().season_id;
		SeasonStats::<T>::mutate(season_id, &from, |stats| {
			stats.minted = 1;
			stats.forged = 1;
		});
		PlayerConfigs::<T>::mutate(&from, |account| account.free_mints = MintCount::MAX);
	}: _(RawOrigin::Signed(from.clone()), to.clone(), how_many)
	verify {
		assert_last_event::<T>(Event::FreeMintsTransferred { from, to, how_many })
	}

	set_price {
		let name = "player";
		let caller = account::<T>(name);
		create_avatars::<T>(caller.clone(), MaxAvatarsPerPlayer::get())?;
		let season_id = CurrentSeasonStatus::<T>::get().season_id;
		let avatar_id = Owners::<T>::get(&caller, season_id)[0];
		let price = BalanceOf::<T>::unique_saturated_from(u128::MAX);
	}: _(RawOrigin::Signed(caller), avatar_id, price)
	verify {
		assert_last_event::<T>(Event::AvatarPriceSet { avatar_id, price })
	}

	remove_price {
		let name = "player";
		let caller = account::<T>(name);
		create_avatars::<T>(caller.clone(), MaxAvatarsPerPlayer::get())?;
		let season_id = CurrentSeasonStatus::<T>::get().season_id;
		let avatar_id = Owners::<T>::get(&caller, season_id)[0];
		Trade::<T>::insert(season_id, avatar_id, BalanceOf::<T>::unique_saturated_from(u128::MAX));
	}: _(RawOrigin::Signed(caller), avatar_id)
	verify {
		assert_last_event::<T>(Event::AvatarPriceUnset { avatar_id })
	}

	buy {
		let (buyer_name, seller_name) = ("buyer", "seller");
		let (buyer, seller) = (account::<T>(buyer_name), account::<T>(seller_name));
		let n in 1 .. MaxAvatarsPerPlayer::get();
		create_avatars::<T>(buyer.clone(), n - 1)?;
		create_avatars::<T>(seller.clone(), n)?;

		let sell_fee = BalanceOf::<T>::unique_saturated_from(u64::MAX / 2);
		let trade_fee = sell_fee / BalanceOf::<T>::unique_saturated_from(100_u8);
		CurrencyOf::<T>::make_free_balance_be(&buyer, sell_fee + trade_fee);
		CurrencyOf::<T>::make_free_balance_be(&seller, sell_fee);

		let season_id = CurrentSeasonStatus::<T>::get().season_id;
		let avatar_id = Owners::<T>::get(&seller, season_id)[0];
		Trade::<T>::insert(season_id, avatar_id, sell_fee);
	}: _(RawOrigin::Signed(buyer.clone()), avatar_id)
	verify {
		assert_last_event::<T>(Event::AvatarTraded { avatar_id, from: seller, to: buyer, price: sell_fee })
	}

	upgrade_storage {
		create_seasons::<T>(1)?;
		let player = account::<T>("player");
		let current_season_id = CurrentSeasonStatus::<T>::get().season_id;
		let season = Seasons::<T>::get(current_season_id).unwrap();
		CurrencyOf::<T>::make_free_balance_be(&player, season.fee.upgrade_storage);
	}: _(RawOrigin::Signed(player.clone()), None, None)
	verify {
		assert_last_event::<T>(Event::StorageTierUpgraded {
			account: player, season_id: current_season_id,
		})
	}

	set_organizer {
		let organizer = account::<T>("organizer");
	}: _(RawOrigin::Root, organizer.clone())
	verify {
		assert_last_event::<T>(Event::<T>::OrganizerSet { organizer })
	}

	set_treasurer {
		let season_id = 369;
		let treasurer = account::<T>("treasurer");
	}: _(RawOrigin::Root, season_id, treasurer.clone())
	verify {
		assert_last_event::<T>(Event::TreasurerSet { season_id, treasurer })
	}

	claim_treasury {
		create_seasons::<T>(3)?;
		let season_id = 1;
		let treasurer = account::<T>("treasurer");
		let amount = 1_000_000_000_000_u64.unique_saturated_into();
		Treasurer::<T>::insert(season_id, treasurer.clone());
		Treasury::<T>::mutate(season_id, |balance| balance.saturating_accrue(amount));
		CurrencyOf::<T>::deposit_creating(&AAvatars::<T>::treasury_account_id(), amount);
		CurrencyOf::<T>::make_free_balance_be(&treasurer, CurrencyOf::<T>::minimum_balance());
	}: _(RawOrigin::Signed(treasurer.clone()), season_id)
	verify {
		assert_last_event::<T>(Event::TreasuryClaimed { season_id, treasurer, amount })
	}

	set_season {
		let organizer = account::<T>("organizer");
		Organizer::<T>::put(&organizer);

		let season_id = 1;
		let season = Season {
			max_tier_forges: u32::MAX,
			max_variations: 15,
			max_components: 16,
			min_sacrifices: SacrificeCount::MAX,
			max_sacrifices: SacrificeCount::MAX,
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
			base_prob: 99,
			per_period: BlockNumberFor::<T>::from(1_u32),
			periods: u16::MAX,
			fee: Fee {
				mint: MintFees {
					one: BalanceOf::<T>::unique_saturated_from(u128::MAX),
					three: BalanceOf::<T>::unique_saturated_from(u128::MAX),
					six: BalanceOf::<T>::unique_saturated_from(u128::MAX),
				},
				transfer_avatar: BalanceOf::<T>::unique_saturated_from(u128::MAX),
				buy_minimum: BalanceOf::<T>::unique_saturated_from(u128::MAX),
				buy_percent: u8::MAX,
				upgrade_storage: BalanceOf::<T>::unique_saturated_from(u128::MAX),
				prepare_avatar: BalanceOf::<T>::unique_saturated_from(u128::MAX),
				set_price_unlock: BalanceOf::<T>::unique_saturated_from(u128::MAX),
				avatar_transfer_unlock: BalanceOf::<T>::unique_saturated_from(u128::MAX),
			},
			mint_logic: LogicGeneration::First,
			forge_logic: LogicGeneration::First,
		};
		let season_meta = SeasonMeta {
			name: [u8::MAX; 100].to_vec().try_into().unwrap(),
			description: [u8::MAX; 1_000].to_vec().try_into().unwrap(),
		};
		let season_schedule = SeasonSchedule {
			early_start: BlockNumberFor::<T>::from(u32::MAX - 2),
			start: BlockNumberFor::<T>::from(u32::MAX - 1),
			end: BlockNumberFor::<T>::from(u32::MAX),
		};
		let trade_filters = TradeFilters(sp_runtime::BoundedVec::try_from(vec![
			u32::from_le_bytes([0x11, 0x07, 0x00, 0x00]), // CrazyDude pet
			u32::from_le_bytes([0x12, 0x36, 0x00, 0x00]), // GiantWoodStick armor front pet part
			u32::from_le_bytes([0x25, 0x07, 0x00, 0xFF]), // Metals of quantity 255
			u32::from_le_bytes([0x25, 0x02, 0x00, 0x00]), // Electronics of any quantity
			u32::from_le_bytes([0x30, 0x00, 0x00, 0x00]), // Any Essence
			u32::from_le_bytes([0x41, 0x00, 0x00, 0xF0]), // ArmorBase of quantity 240
			u32::from_le_bytes([0x45, 0x00, 0x00, 0x0F]), // WeaponVersion1 of quantity 15
		]).expect("Should create vec"));
	}: _(RawOrigin::Signed(organizer), season_id, Some(season.clone()), Some(season_meta.clone()), Some(season_schedule.clone()), Some(trade_filters.clone()))
	verify {
		assert_last_event::<T>(Event::UpdatedSeason {
			season_id, season: Some(season), meta: Some(season_meta), schedule: Some(season_schedule), trade_filters: Some(trade_filters)
		})
	}

	update_global_config {
		let organizer = account::<T>("organizer");
		Organizer::<T>::put(&organizer);

		let config = GlobalConfig {
			mint: MintConfig {
				open: true,
				cooldown: BlockNumberFor::<T>::from(u32::MAX),
				free_mint_fee_multiplier: MintCount::MAX,
			},
			forge: ForgeConfig { open: true },
			avatar_transfer: AvatarTransferConfig {
				open: true,
			},
			freemint_transfer: FreemintTransferConfig {
				mode: FreeMintTransferMode::Open,
				free_mint_transfer_fee: MintCount::MAX,
				min_free_mint_transfer: MintCount::MAX,
			},
			trade: TradeConfig { open: true },
			nft_transfer: NftTransferConfig { open: true },
			affiliate_config: AffiliateConfig::default(),
		};
	}: _(RawOrigin::Signed(organizer), config.clone())
	verify {
		assert_last_event::<T>(Event::UpdatedGlobalConfig(config))
	}

	set_free_mints {
		let organizer = account::<T>("organizer");
		Organizer::<T>::put(&organizer);

		let target = account::<T>("target");
		let how_many = MintCount::MAX;
	}: _(RawOrigin::Signed(organizer), target.clone(), how_many)
	verify {
		assert_last_event::<T>(Event::FreeMintsSet { target, how_many });
	}

	lock_avatar {
		let name = "player";
		let player = account::<T>(name);
		let n in 1 .. MaxAvatarsPerPlayer::get();
		create_avatars::<T>(player.clone(), n)?;

		let season_id = CurrentSeasonStatus::<T>::get().season_id;
		let avatar_ids = Owners::<T>::get(&player, season_id);
		let avatar_id = avatar_ids[avatar_ids.len() - 1];

		let organizer = account::<T>("organizer");
	}: _(RawOrigin::Signed(player), avatar_id)
	verify {
		assert_last_event::<T>(Event::AvatarLocked { avatar_id })
	}

	unlock_avatar {
		let name = "player";
		let player = account::<T>(name);
		let n in 1 .. MaxAvatarsPerPlayer::get();
		create_avatars::<T>(player.clone(), n)?;

		let season_id = CurrentSeasonStatus::<T>::get().season_id;
		let avatar_ids = Owners::<T>::get(&player, season_id);
		let avatar_id = avatar_ids[avatar_ids.len() - 1];

		let organizer = account::<T>("organizer");

		AAvatars::<T>::lock_avatar(RawOrigin::Signed(player.clone()).into(), avatar_id)?;
	}: _(RawOrigin::Signed(player), avatar_id)
	verify {
		assert_last_event::<T>(Event::AvatarUnlocked { avatar_id })
	}

	impl_benchmark_test_suite!(
		Pallet,
		crate::mock::new_test_ext(),
		crate::mock::Runtime
	);
}
