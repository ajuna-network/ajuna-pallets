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
	mock::{
		Balances, MockAccountId, MockAffiliatesFeeProvider, MockFilterHandler, MockSeasonManager,
		MockTournamentFeeProvider, MockTransitionFeeProvider, MockTreasuryManager, RuntimeEvent,
		RuntimeOrigin, SageBenchmarkHelper, System, Test, SEASON_ID_0,
	},
	Pallet as Sage, *,
};
use ajuna_primitives::{asset_manager::AssetInspector, fee_handler::GameFeeHandler};
use example_transition::{
	generic::ExampleTransitionGeneric,
	types::{Asset, AssetId},
};
use frame_benchmarking::benchmarks_instance_pallet;
use frame_support::parameter_types;
use frame_system::RawOrigin;
use sp_runtime::BuildStorage;

parameter_types! {
	pub const BenchPalletId: PalletId = PalletId(*b"sage/bch");
}

pub struct MockAssetMediatorBench;

impl AssetManager for MockAssetMediatorBench {
	type AccountId = MockAccountId;
	type AssetId = AssetId;
	type Asset = Asset;

	fn ensure_ownership(
		owner: &Self::AccountId,
		asset_id: &Self::AssetId,
	) -> Result<Self::Asset, DispatchError> {
		<Sage<Test, ()> as AssetManager>::ensure_ownership(owner, asset_id)
	}

	fn lock_asset(
		lock_id: LockIdentifier,
		owner: Self::AccountId,
		asset_id: Self::AssetId,
	) -> Result<Self::Asset, DispatchError> {
		<Sage<Test, ()> as AssetManager>::lock_asset(lock_id, owner, asset_id)
	}

	fn unlock_asset(
		lock_id: LockIdentifier,
		owner: Self::AccountId,
		asset_id: Self::AssetId,
	) -> Result<Self::Asset, DispatchError> {
		<Sage<Test, ()> as AssetManager>::unlock_asset(lock_id, owner, asset_id)
	}

	fn is_locked(asset: &Self::AssetId) -> Option<Lock<Self::AccountId>> {
		<Sage<Test, ()> as AssetManager>::is_locked(asset)
	}

	fn nft_transfer_open() -> bool {
		<Sage<Test, ()> as AssetManager>::nft_transfer_open()
	}

	fn handle_asset_prepare_fee(
		asset: &Self::Asset,
		from: &Self::AccountId,
		fees_recipient: &Self::AccountId,
	) -> Result<(), DispatchError> {
		<Sage<Test, ()> as AssetManager>::handle_asset_prepare_fee(asset, from, fees_recipient)
	}
}

impl AssetInspector for MockAssetMediatorBench {
	type AssetId = AssetId;
	type Asset = Asset;

	fn get_asset(asset_id: &Self::AssetId) -> Result<Self::Asset, DispatchError> {
		<Sage<Test, ()> as AssetInspector>::get_asset(asset_id)
	}
}

impl Config for Test {
	type PalletId = BenchPalletId;
	type SageGameTransition = ExampleTransitionGeneric<MockAccountId, MockAssetMediatorBench>;
	type SeasonHandler = MockSeasonManager;
	type FeeHandler = GameFeeHandler<
		MockAccountId,
		Balances,
		MockAffiliatesFeeProvider,
		MockTournamentFeeProvider,
		MockTransitionFeeProvider,
		MockTreasuryManager,
	>;
	type FilterHandler = MockFilterHandler;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type BenchmarkHelper = SageBenchmarkHelper;
}

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

fn create_asset_for<T: Config<I>, I: 'static>(
	account: &T::AccountId,
	season_id: &SeasonIdOf<T, I>,
	seed: u32,
) -> AssetIdOf<T, I> {
	let (asset_id, asset) = T::BenchmarkHelper::create_asset(seed);

	T::SeasonHandler::register_asset_in(&asset_id, season_id).expect("Asset should be registered");
	Assets::<T, I>::insert(&asset_id, (account, asset));
	AssetOwners::<T, I>::insert((account, season_id, &asset_id), ());

	asset_id
}

benchmarks_instance_pallet! {
	set_organizer {
		let acc_2 = account::<T, I>(ACC_2);
	}: _(RawOrigin::Root, acc_2.clone())
	verify {
		assert_last_event::<T, I>(Event::OrganizerSet { organizer: acc_2 })
	}

	update_general_config {
		let acc_1 = account::<T, I>(ACC_1);
		let general_config = GeneralConfigOf::<T, I> {
			transition: T::BenchmarkHelper::create_transition_config(0),
			transfer: Default::default(),
			trade: Default::default()
		};
	}: _(RawOrigin::Signed(acc_1), general_config.clone())
	verify {
		assert_last_event::<T, I>(Event::UpdatedGeneralConfig { updated_config: general_config })
	}

	update_unlock_rule {
		let acc_1 = account::<T, I>(ACC_1);
		let season_id = T::BenchmarkHelper::create_season_id(SEASON_ID_0 as u32);
		let feature = LockableFeature::TradeAsset;
		let unlock_rule: UnlockRule = [10, 10, 10, 10, 10];
		// Benchmark helper season_id
	}: _(RawOrigin::Signed(acc_1), season_id.clone(), feature, unlock_rule)
	verify {
		assert_last_event::<T, I>(Event::UpdatedUnlockRule {
			season_id,
			feature,
			updated_rule: unlock_rule,
		})
	}

	upgrade_asset_inventory {
		let acc_1 = account::<T, I>(ACC_1);
		let acc_2 = account::<T, I>(ACC_2);
		let season_id = T::BenchmarkHelper::create_season_id(SEASON_ID_0 as u32);
		let in_season = Some(season_id.clone());
		// Benchmark helper season_id
	}: _(RawOrigin::Signed(acc_1), Some(acc_2.clone()), in_season)
	verify {
		assert_last_event::<T, I>(Event::InventoryTierUpgraded {
			account: acc_2,
			season_id,
			new_tier: InventoryTier::Two,
		})
	}

	update_asset_trade_filter {
		let acc_1 = account::<T, I>(ACC_1);
		let season_id = T::BenchmarkHelper::create_season_id(SEASON_ID_0 as u32);
		let filter = T::BenchmarkHelper::create_asset_trade_filter(0);
		let trade_filter = AssetFilterOf::<T,I>::Trade(filter.clone());
	}: update_asset_filter(RawOrigin::Signed(acc_1), season_id.clone(), trade_filter)
	verify {
		assert_last_event::<T, I>(Event::UpdatedTradeFilter {
			season_id,
			filter,
		})
	}

	update_asset_transfer_filter {
		let acc_1 = account::<T, I>(ACC_1);
		let season_id = T::BenchmarkHelper::create_season_id(SEASON_ID_0 as u32);
		let filter = T::BenchmarkHelper::create_asset_transfer_filter(0);
		let transfer_filter = AssetFilterOf::<T,I>::Transfer(filter.clone());
	}: update_asset_filter(RawOrigin::Signed(acc_1), season_id.clone(), transfer_filter)
	verify {
		assert_last_event::<T, I>(Event::UpdatedTransferFilter {
			season_id,
			filter,
		})
	}

	transfer_asset {
		let acc_1 = account::<T, I>(ACC_1);
		let acc_2 = account::<T, I>(ACC_2);
		let season_id = T::BenchmarkHelper::create_season_id(SEASON_ID_0 as u32);
		let asset_id = create_asset_for::<T, I>(&acc_1, &season_id, 2);
	}: _(RawOrigin::Signed(acc_1.clone()), acc_2.clone(), asset_id.clone())
	verify {
		assert_last_event::<T, I>(Event::AssetTransferred {
			from: acc_1,
			to: acc_2,
			asset_id,
		})
	}

	set_asset_price {
		let acc_1 = account::<T, I>(ACC_1);
		let season_id = T::BenchmarkHelper::create_season_id(SEASON_ID_0 as u32);
		let asset_id = create_asset_for::<T, I>(&acc_1, &season_id, 31);
		let price = 45_242_u32;
	}: _(RawOrigin::Signed(acc_1), asset_id.clone(), price.into())
	verify {
		assert_last_event::<T, I>(Event::AssetPriceSet {
			asset_id,
			price: price.into(),
		})
	}

	remove_asset_price {
		let acc_1 = account::<T, I>(ACC_1);
		let season_id = T::BenchmarkHelper::create_season_id(SEASON_ID_0 as u32);
		let asset_id = create_asset_for::<T, I>(&acc_1, &season_id, 31);
		let price = 45_242_u32;
		Sage::<T, I>::set_asset_price(
			RawOrigin::Signed(acc_1.clone()).into(),
			asset_id.clone(),
			price.into())
		.expect("Should set price");
	}: _(RawOrigin::Signed(acc_1), asset_id.clone())
	verify {
		assert_last_event::<T, I>(Event::AssetPriceUnset {
			asset_id,
		})
	}

	buy_asset {
		let acc_1 = account::<T, I>(ACC_1);
		let acc_2 = account::<T, I>(ACC_2);
		let season_id = T::BenchmarkHelper::create_season_id(SEASON_ID_0 as u32);
		let asset_id = create_asset_for::<T, I>(&acc_1, &season_id, 31);
		let price = 45_242_u32;
		Sage::<T, I>::set_asset_price(
			RawOrigin::Signed(acc_1.clone()).into(),
			asset_id.clone(),
			price.into())
		.expect("Should set price");
	}: _(RawOrigin::Signed(acc_2.clone()), asset_id.clone())
	verify {
		assert_last_event::<T, I>(Event::AssetTraded {
			asset_id,
			from: acc_1,
			to: acc_2,
			price: price.into(),
		})
	}

	lock_asset {
		let acc_1 = account::<T, I>(ACC_1);
		let season_id = T::BenchmarkHelper::create_season_id(SEASON_ID_0 as u32);
		let asset_id = create_asset_for::<T, I>(&acc_1, &season_id, 31);
		let expected_lock = Lock::new(*SAGE_LOCK_ID, acc_1.clone());
	}: _(RawOrigin::Signed(acc_1), asset_id.clone())
	verify {
		assert_last_event::<T, I>(Event::AssetLocked {
			asset_id,
			lock: expected_lock,
		})
	}

	unlock_asset {
		let acc_1 = account::<T, I>(ACC_1);
		let season_id = T::BenchmarkHelper::create_season_id(SEASON_ID_0 as u32);
		let asset_id = create_asset_for::<T, I>(&acc_1, &season_id, 31);
		let expected_lock = Lock::new(*SAGE_LOCK_ID, acc_1.clone());
		Sage::<T, I>::lock_asset(
			RawOrigin::Signed(acc_1.clone()).into(),
			asset_id.clone())
		.expect("Should lock asset");
	}: _(RawOrigin::Signed(acc_1), asset_id.clone())
	verify {
		assert_last_event::<T, I>(Event::AssetUnlocked {
			asset_id,
			lock: expected_lock,
		})
	}

	unlock_trade_asset_feature {
		let acc_1 = account::<T, I>(ACC_1);
		let target = UnlockTarget::OneselfFree;
		let feature = LockableFeature::TradeAsset;
		let season_id = T::BenchmarkHelper::create_season_id(SEASON_ID_0 as u32);
	}: unlock_feature(RawOrigin::Signed(acc_1.clone()), target, feature, season_id.clone())
	verify {
		assert_last_event::<T, I>(Event::FeatureUnlocked {
			feature,
			season_id,
			account: acc_1,
		})
	}

	unlock_transfer_asset_feature {
		let acc_1 = account::<T, I>(ACC_1);
		let target = UnlockTarget::OneselfFree;
		let feature = LockableFeature::TransferAsset;
		let season_id = T::BenchmarkHelper::create_season_id(SEASON_ID_0 as u32);
	}: unlock_feature(RawOrigin::Signed(acc_1.clone()), target, feature, season_id.clone())
	verify {
		assert_last_event::<T, I>(Event::FeatureUnlocked {
			feature,
			season_id,
			account: acc_1,
		})
	}

	state_transition {
		let acc_1 = account::<T, I>(ACC_1);
		let season_id = T::BenchmarkHelper::create_season_id(SEASON_ID_0 as u32);
		let asset_id_1 = create_asset_for::<T, I>(&acc_1, &season_id, 31);
		let asset_id_2 = create_asset_for::<T, I>(&acc_1, &season_id, 124);
		let asset_id_3 = create_asset_for::<T, I>(&acc_1, &season_id, 482);
		let asset_id_4 = create_asset_for::<T, I>(&acc_1, &season_id, 1);
		let asset_id_5 = create_asset_for::<T, I>(&acc_1, &season_id, 73);
		let extra = T::BenchmarkHelper::create_extra(0);
		let transition_id = T::BenchmarkHelper::create_transition_id(99);
		let asset_ids = vec![asset_id_1, asset_id_2, asset_id_3, asset_id_4, asset_id_5];
	}: _(RawOrigin::Signed(acc_1.clone()), transition_id.clone(), asset_ids, extra)
	verify {
		assert_last_event::<T, I>(Event::TransitionExecuted {
			account: acc_1,
			id: transition_id,
		})
	}

	impl_benchmark_test_suite!(
		Sage,
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
