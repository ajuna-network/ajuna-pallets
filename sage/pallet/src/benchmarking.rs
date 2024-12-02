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

use crate::{
	config::{InventoryTier, Locks},
	mock::{Balances, RuntimeOrigin, SageBenchmarkHelper, System, Test, SEASON_ID_0},
	pallet::AssetFilterOf,
	AssetTradePrices, BalanceOf, BenchmarkHelper, Call, Config, Event, GeneralConfigOf,
	GeneralConfigStore, LockableFeature, Organizer, Pallet, PlayerSeasonConfigs, SeasonUnlocks,
	UnlockRule, UnlockTarget, SAGE_LOCK_ID,
};
use ajuna_primitives::asset_manager::Lock;
use frame_benchmarking::benchmarks;
use frame_system::RawOrigin;
use sp_runtime::BuildStorage;

const ACC_1: &str = "acc_1";
const ACC_2: &str = "acc_2";

fn account<T: Config<I>, I: 'static>(name: &'static str) -> T::AccountId {
	let index = 0;
	let seed = 0;
	frame_benchmarking::account(name, index, seed)
}

fn assert_last_event<T: Config<I>, I: 'static>(avatars_event: Event<T, I>) {
	let event = <T as Config<I>>::RuntimeEvent::from(avatars_event);
	frame_system::Pallet::<T>::assert_last_event(event.into());
}

benchmarks! {
	set_organizer {
		let acc_2 = account::<T, ()>(ACC_2);
	}: _(RawOrigin::Root, acc_2.clone())
	verify {
		assert_last_event::<T, ()>(Event::OrganizerSet { organizer: acc_2 })
	}

	update_general_config {
		let acc_1 = account::<T, ()>(ACC_1);
		let general_config = GeneralConfigOf::<T, ()> {
			transition: T::BenchmarkHelper::create_transition_config(0),
			transfer: Default::default(),
			trade: Default::default()
		};
	}: _(RawOrigin::Signed(acc_1), general_config.clone())
	verify {
		assert_last_event::<T, ()>(Event::UpdatedGeneralConfig { updated_config: general_config })
	}

	update_unlock_rule {
		let acc_1 = account::<T, ()>(ACC_1);
		let season_id = T::BenchmarkHelper::create_season_id(SEASON_ID_0 as u32);
		let feature = LockableFeature::TradeAsset;
		let unlock_rule: UnlockRule = [10, 10, 10, 10, 10];
		// Benchmark helper season_id
	}: _(RawOrigin::Signed(acc_1), season_id.clone(), feature, unlock_rule)
	verify {
		assert_last_event::<T, ()>(Event::UpdatedUnlockRule {
			season_id,
			feature,
			updated_rule: unlock_rule,
		})
	}

	upgrade_asset_inventory {
		let acc_1 = account::<T, ()>(ACC_1);
		let acc_2 = account::<T, ()>(ACC_2);
		let season_id = T::BenchmarkHelper::create_season_id(SEASON_ID_0 as u32);
		let in_season = Some(season_id.clone());
		// Benchmark helper season_id
	}: _(RawOrigin::Signed(acc_1), Some(acc_2.clone()), in_season)
	verify {
		assert_last_event::<T, ()>(Event::InventoryTierUpgraded {
			account: acc_2,
			season_id,
			new_tier: InventoryTier::Two,
		})
	}

	update_asset_trade_filter {
		let acc_1 = account::<T, ()>(ACC_1);
		let season_id = T::BenchmarkHelper::create_season_id(SEASON_ID_0 as u32);
		let filter = T::BenchmarkHelper::create_asset_trade_filter(0);
		let trade_filter = AssetFilterOf::<T, ()>::Trade(filter.clone());
	}: update_asset_filter(RawOrigin::Signed(acc_1), season_id.clone(), trade_filter)
	verify {
		assert_last_event::<T, ()>(Event::UpdatedTradeFilter {
			season_id,
			filter,
		})
	}

	update_asset_transfer_filter {
		let acc_1 = account::<T, ()>(ACC_1);
		let season_id = T::BenchmarkHelper::create_season_id(SEASON_ID_0 as u32);
		let filter = T::BenchmarkHelper::create_asset_transfer_filter(0);
		let transfer_filter = AssetFilterOf::<T, ()>::Transfer(filter.clone());
	}: update_asset_filter(RawOrigin::Signed(acc_1), season_id.clone(), transfer_filter)
	verify {
		assert_last_event::<T, ()>(Event::UpdatedTransferFilter {
			season_id,
			filter,
		})
	}

	transfer_asset {
		let acc_1 = account::<T, ()>(ACC_1);
		let acc_2 = account::<T, ()>(ACC_2);
		let season_id = T::BenchmarkHelper::create_season_id(SEASON_ID_0 as u32);
		let asset_id = SageBenchmarkHelper::create_asset_for::<T, ()>(&acc_1, &season_id, 2);
	}: _(RawOrigin::Signed(acc_1.clone()), acc_2.clone(), asset_id.clone())
	verify {
		assert_last_event::<T, ()>(Event::AssetTransferred {
			from: acc_1,
			to: acc_2,
			asset_id,
		})
	}

	set_asset_price {
		let acc_1 = account::<T, ()>(ACC_1);
		let season_id = T::BenchmarkHelper::create_season_id(SEASON_ID_0 as u32);
		let asset_id = SageBenchmarkHelper::create_asset_for::<T, ()>(&acc_1, &season_id, 31);
		let price = 45_242_u32;
	}: _(RawOrigin::Signed(acc_1), asset_id.clone(), price.into())
	verify {
		assert_last_event::<T, ()>(Event::AssetPriceSet {
			asset_id,
			price: price.into(),
		})
	}

	remove_asset_price {
		let acc_1 = account::<T, ()>(ACC_1);
		let season_id = T::BenchmarkHelper::create_season_id(SEASON_ID_0 as u32);
		let asset_id = SageBenchmarkHelper::create_asset_for::<T, ()>(&acc_1, &season_id, 31);
		let price = BalanceOf::<T, ()>::from(45_242_u32);
		AssetTradePrices::<T, ()>::insert(&season_id, &asset_id, &price);
	}: _(RawOrigin::Signed(acc_1), asset_id.clone())
	verify {
		assert_last_event::<T, ()>(Event::AssetPriceUnset {
			asset_id,
		})
	}

	buy_asset {
		let acc_1 = account::<T, ()>(ACC_1);
		let acc_2 = account::<T, ()>(ACC_2);
		let season_id = T::BenchmarkHelper::create_season_id(SEASON_ID_0 as u32);
		let asset_id = SageBenchmarkHelper::create_asset_for::<T, ()>(&acc_1, &season_id, 31);
		let price = BalanceOf::<T, ()>::from(45_242_u32);
		AssetTradePrices::<T, ()>::insert(&season_id, &asset_id, &price);
	}: _(RawOrigin::Signed(acc_2.clone()), asset_id.clone())
	verify {
		assert_last_event::<T, ()>(Event::AssetTraded {
			asset_id,
			from: acc_1,
			to: acc_2,
			price: price.into(),
		})
	}

	lock_asset {
		let acc_1 = account::<T, ()>(ACC_1);
		let season_id = T::BenchmarkHelper::create_season_id(SEASON_ID_0 as u32);
		let asset_id = SageBenchmarkHelper::create_asset_for::<T, ()>(&acc_1, &season_id, 31);
		let expected_lock = Lock::new(*SAGE_LOCK_ID, acc_1.clone());
	}: _(RawOrigin::Signed(acc_1), asset_id.clone())
	verify {
		assert_last_event::<T, ()>(Event::AssetLocked {
			asset_id,
			lock: expected_lock,
		})
	}

	unlock_asset {
		let acc_1 = account::<T, ()>(ACC_1);
		let season_id = T::BenchmarkHelper::create_season_id(SEASON_ID_0 as u32);
		let asset_id = SageBenchmarkHelper::create_asset_for::<T, ()>(&acc_1, &season_id, 31);
		let expected_lock = Lock::new(*SAGE_LOCK_ID, acc_1.clone());
		Pallet::<T, ()>::lock_asset(
			RawOrigin::Signed(acc_1.clone()).into(),
			asset_id.clone())
		.expect("Should lock asset");
	}: _(RawOrigin::Signed(acc_1), asset_id.clone())
	verify {
		assert_last_event::<T, ()>(Event::AssetUnlocked {
			asset_id,
			lock: expected_lock,
		})
	}

	unlock_trade_asset_feature {
		let acc_1 = account::<T, ()>(ACC_1);
		let target = UnlockTarget::OneselfFree;
		let feature = LockableFeature::TradeAsset;
		let season_id = T::BenchmarkHelper::create_season_id(SEASON_ID_0 as u32);
	}: unlock_feature(RawOrigin::Signed(acc_1.clone()), target, feature, season_id.clone())
	verify {
		assert_last_event::<T, ()>(Event::FeatureUnlocked {
			feature,
			season_id,
			account: acc_1,
		})
	}

	unlock_transfer_asset_feature {
		let acc_1 = account::<T, ()>(ACC_1);
		let target = UnlockTarget::OneselfFree;
		let feature = LockableFeature::TransferAsset;
		let season_id = T::BenchmarkHelper::create_season_id(SEASON_ID_0 as u32);
	}: unlock_feature(RawOrigin::Signed(acc_1.clone()), target, feature, season_id.clone())
	verify {
		assert_last_event::<T, ()>(Event::FeatureUnlocked {
			feature,
			season_id,
			account: acc_1,
		})
	}

	state_transition {
		let acc_1 = account::<T, ()>(ACC_1);
		let season_id = T::BenchmarkHelper::create_season_id(SEASON_ID_0 as u32);
		let (transition_id, asset_ids) = T::BenchmarkHelper::create_bench_transition_for(&acc_1, &season_id, 99);
		let extra = T::BenchmarkHelper::create_extra(0);
	}: _(RawOrigin::Signed(acc_1.clone()), transition_id.clone(), asset_ids, extra)
	verify {
		assert_last_event::<T, ()>(Event::TransitionExecuted {
			account: acc_1,
			id: transition_id,
		})
	}

	impl_benchmark_test_suite!(
		Pallet,
		new_test_ext(),
		Test
	);
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext.execute_with(|| {
		GeneralConfigStore::<Test, ()>::mutate(|config| {
			config.trade.open = true;
			config.transfer.open = true;
		});
		SeasonUnlocks::<Test, ()>::mutate(SEASON_ID_0, LockableFeature::TradeAsset, |rule| {
			*rule = Some(UnlockRule::from([0, 0, 0, 0, 0]));
		});
		SeasonUnlocks::<Test, ()>::mutate(SEASON_ID_0, LockableFeature::TransferAsset, |rule| {
			*rule = Some(UnlockRule::from([0, 0, 0, 0, 0]));
		});

		let acc_1 = account::<Test, ()>(ACC_1);
		Organizer::<Test, ()>::put(acc_1);
		PlayerSeasonConfigs::<Test, ()>::mutate(acc_1, SEASON_ID_0, |config| {
			config.locks = Locks::all_unlocked();
		});
		Balances::force_set_balance(RuntimeOrigin::root(), acc_1, 100_000)
			.expect("Should set balance");

		let acc_2 = account::<Test, ()>(ACC_2);
		PlayerSeasonConfigs::<Test, ()>::mutate(acc_2, SEASON_ID_0, |config| {
			config.locks = Locks::all_unlocked();
		});
		Balances::force_set_balance(RuntimeOrigin::root(), acc_2, 100_000)
			.expect("Should set balance");
	});
	ext
}
