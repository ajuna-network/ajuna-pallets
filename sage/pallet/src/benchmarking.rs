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

use crate::{
	config::{InventoryTier, Locks},
	pallet::{AssetFilterOf, TradeFilterOf, TransferFilterOf},
	AssetTradePrices, BalanceOf, BenchmarkHelper, Call, Config, Event, ExtraOf, GeneralConfigOf,
	GeneralConfigStore, LockableFeature, Organizer, Pallet, PlayerSeasonConfigs, SeasonIdOf,
	SeasonUnlocks, UnlockRule, UnlockTarget, SAGE_LOCK_ID,
};
use ajuna_primitives::{asset_manager::Lock, season_manager::SeasonManager};
use frame_benchmarking::v2::*;
use frame_support::traits::Currency;
use frame_system::RawOrigin;

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

fn setup_organizer<T: Config<I>, I: 'static>(organizer: T::AccountId) {
	Organizer::<T, I>::set(Some(organizer));
}

fn set_account_balance<T: Config<I>, I: 'static>(account: &T::AccountId, balance: BalanceOf<T, I>) {
	let _ = T::Currency::deposit_creating(account, balance);
}

fn unlock_season_features_for<T: Config<I>, I: 'static>(season_id: &SeasonIdOf<T, I>) {
	GeneralConfigStore::<T, I>::mutate(|config| {
		config.trade.open = true;
		config.transfer.open = true;
	});
	SeasonUnlocks::<T, I>::mutate(season_id, LockableFeature::TradeAsset, |rule| {
		*rule = Some(UnlockRule::from([0, 0, 0, 0, 0]));
	});
	SeasonUnlocks::<T, I>::mutate(season_id, LockableFeature::TransferAsset, |rule| {
		*rule = Some(UnlockRule::from([0, 0, 0, 0, 0]));
	});
}

fn unlock_player_features_for<T: Config<I>, I: 'static>(
	account: &T::AccountId,
	season_id: &SeasonIdOf<T, I>,
) {
	PlayerSeasonConfigs::<T, I>::mutate(account, season_id, |config| {
		config.locks = Locks::all_unlocked();
	});
}

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn set_organizer() {
		let acc_2 = account::<T, ()>(ACC_2);

		#[extrinsic_call]
		_(RawOrigin::Root, acc_2.clone());

		assert_last_event::<T, ()>(Event::OrganizerSet { organizer: acc_2 });
	}

	#[benchmark]
	fn update_general_config() {
		let acc_1 = account::<T, ()>(ACC_1);
		setup_organizer::<T, ()>(acc_1.clone());
		let general_config = GeneralConfigOf::<T, ()>::default();

		#[extrinsic_call]
		_(RawOrigin::Signed(acc_1), general_config.clone());

		assert_last_event::<T, ()>(Event::UpdatedGeneralConfig { updated_config: general_config });
	}

	#[benchmark]
	fn update_unlock_rule() {
		let acc_1 = account::<T, ()>(ACC_1);
		setup_organizer::<T, ()>(acc_1.clone());
		let season_id = <T as Config<()>>::SeasonHandler::get_current_season_id()
			.expect("Should get current season");
		let feature = LockableFeature::TradeAsset;
		let unlock_rule: UnlockRule = [10, 10, 10, 10, 10];

		#[extrinsic_call]
		_(RawOrigin::Signed(acc_1), season_id.clone(), feature, unlock_rule);

		assert_last_event::<T, ()>(Event::UpdatedUnlockRule {
			season_id,
			feature,
			updated_rule: unlock_rule,
		});
	}

	#[benchmark]
	fn upgrade_asset_inventory() {
		let acc_1 = account::<T, ()>(ACC_1);
		setup_organizer::<T, ()>(acc_1.clone());
		set_account_balance::<T, ()>(&acc_1, 100_u32.into());
		let acc_2 = account::<T, ()>(ACC_2);
		let season_id = <T as Config<()>>::SeasonHandler::get_current_season_id()
			.expect("Should get current season");
		let in_season = Some(season_id.clone());
		let payment = T::BenchmarkHelper::create_payment_kind();

		#[extrinsic_call]
		_(RawOrigin::Signed(acc_1), Some(acc_2.clone()), in_season, Some(payment));

		assert_last_event::<T, ()>(Event::InventoryTierUpgraded {
			account: acc_2,
			season_id,
			new_tier: InventoryTier::Two,
		});
	}

	#[benchmark]
	fn update_asset_trade_filter() {
		let acc_1 = account::<T, ()>(ACC_1);
		setup_organizer::<T, ()>(acc_1.clone());
		let season_id = <T as Config<()>>::SeasonHandler::get_current_season_id()
			.expect("Should get current season");
		let filter = TradeFilterOf::<T, ()>::default();
		let trade_filter = AssetFilterOf::<T, ()>::Trade(filter.clone());

		#[extrinsic_call]
		update_asset_filter(RawOrigin::Signed(acc_1), season_id.clone(), trade_filter);

		assert_last_event::<T, ()>(Event::UpdatedTradeFilter { season_id, filter });
	}

	#[benchmark]
	fn update_asset_transfer_filter() {
		let acc_1 = account::<T, ()>(ACC_1);
		setup_organizer::<T, ()>(acc_1.clone());
		let season_id = <T as Config<()>>::SeasonHandler::get_current_season_id()
			.expect("Should get current season");
		let filter = TransferFilterOf::<T, ()>::default();
		let transfer_filter = AssetFilterOf::<T, ()>::Transfer(filter.clone());

		#[extrinsic_call]
		update_asset_filter(RawOrigin::Signed(acc_1), season_id.clone(), transfer_filter);

		assert_last_event::<T, ()>(Event::UpdatedTransferFilter { season_id, filter });
	}

	#[benchmark]
	fn transfer_asset() {
		let acc_1 = account::<T, ()>(ACC_1);
		set_account_balance::<T, ()>(&acc_1, 100_u32.into());
		let acc_2 = account::<T, ()>(ACC_2);
		let season_id = <T as Config<()>>::SeasonHandler::get_current_season_id()
			.expect("Should get current season");
		unlock_season_features_for::<T, ()>(&season_id);
		unlock_player_features_for::<T, ()>(&acc_1, &season_id);
		let asset_id = T::BenchmarkHelper::create_asset_for(&acc_1, &season_id, 2);
		let payment = T::BenchmarkHelper::create_payment_kind();

		#[extrinsic_call]
		_(RawOrigin::Signed(acc_1.clone()), acc_2.clone(), asset_id.clone(), Some(payment));

		assert_last_event::<T, ()>(Event::AssetTransferred { from: acc_1, to: acc_2, asset_id });
	}

	#[benchmark]
	fn set_asset_price() {
		let acc_1 = account::<T, ()>(ACC_1);
		let season_id = <T as Config<()>>::SeasonHandler::get_current_season_id()
			.expect("Should get current season");
		unlock_season_features_for::<T, ()>(&season_id);
		unlock_player_features_for::<T, ()>(&acc_1, &season_id);
		let asset_id = T::BenchmarkHelper::create_asset_for(&acc_1, &season_id, 31);
		let price = 45_242_u32;

		#[extrinsic_call]
		_(RawOrigin::Signed(acc_1), asset_id.clone(), price.into());

		assert_last_event::<T, ()>(Event::AssetPriceSet { asset_id, price: price.into() });
	}

	#[benchmark]
	fn remove_asset_price() {
		let acc_1 = account::<T, ()>(ACC_1);
		let season_id = <T as Config<()>>::SeasonHandler::get_current_season_id()
			.expect("Should get current season");
		unlock_season_features_for::<T, ()>(&season_id);
		let asset_id = T::BenchmarkHelper::create_asset_for(&acc_1, &season_id, 31);
		let price = BalanceOf::<T, ()>::from(45_242_u32);
		AssetTradePrices::<T, ()>::insert(&season_id, &asset_id, price);

		#[extrinsic_call]
		_(RawOrigin::Signed(acc_1), asset_id.clone());

		assert_last_event::<T, ()>(Event::AssetPriceUnset { asset_id });
	}

	#[benchmark]
	fn buy_asset() {
		let acc_1 = account::<T, ()>(ACC_1);
		let acc_2 = account::<T, ()>(ACC_2);
		set_account_balance::<T, ()>(&acc_2, 100_000_u32.into());
		let season_id = <T as Config<()>>::SeasonHandler::get_current_season_id()
			.expect("Should get current season");
		let asset_id = T::BenchmarkHelper::create_asset_for(&acc_1, &season_id, 31);
		let price = BalanceOf::<T, ()>::from(45_242_u32);
		AssetTradePrices::<T, ()>::insert(&season_id, &asset_id, price);
		let payment = T::BenchmarkHelper::create_payment_kind();

		#[extrinsic_call]
		_(RawOrigin::Signed(acc_2.clone()), asset_id.clone(), Some(payment));

		assert_last_event::<T, ()>(Event::AssetTraded { asset_id, from: acc_1, to: acc_2, price });
	}

	#[benchmark]
	fn lock_asset() {
		let acc_1 = account::<T, ()>(ACC_1);
		let season_id = <T as Config<()>>::SeasonHandler::get_current_season_id()
			.expect("Should get current season");
		let asset_id = T::BenchmarkHelper::create_asset_for(&acc_1, &season_id, 31);
		let expected_lock = Lock::new(*SAGE_LOCK_ID, acc_1.clone());

		#[extrinsic_call]
		_(RawOrigin::Signed(acc_1), asset_id.clone());

		assert_last_event::<T, ()>(Event::AssetLocked { asset_id, lock: expected_lock });
	}

	#[benchmark]
	fn unlock_asset() {
		let acc_1 = account::<T, ()>(ACC_1);
		let season_id = <T as Config<()>>::SeasonHandler::get_current_season_id()
			.expect("Should get current season");
		let asset_id = T::BenchmarkHelper::create_asset_for(&acc_1, &season_id, 31);
		let expected_lock = Lock::new(*SAGE_LOCK_ID, acc_1.clone());
		Pallet::<T, ()>::lock_asset(RawOrigin::Signed(acc_1.clone()).into(), asset_id.clone())
			.expect("Should lock asset");

		#[extrinsic_call]
		_(RawOrigin::Signed(acc_1), asset_id.clone());

		assert_last_event::<T, ()>(Event::AssetUnlocked { asset_id, lock: expected_lock });
	}

	#[benchmark]
	fn unlock_trade_asset_feature() {
		let acc_1 = account::<T, ()>(ACC_1);
		let target = UnlockTarget::OneselfFree;
		let feature = LockableFeature::TradeAsset;
		let season_id = <T as Config<()>>::SeasonHandler::get_current_season_id()
			.expect("Should get current season");
		unlock_season_features_for::<T, ()>(&season_id);
		let payment = T::BenchmarkHelper::create_payment_kind();

		#[extrinsic_call]
		unlock_feature(
			RawOrigin::Signed(acc_1.clone()),
			target,
			feature,
			season_id.clone(),
			Some(payment),
		);

		assert_last_event::<T, ()>(Event::FeatureUnlocked { feature, season_id, account: acc_1 });
	}

	#[benchmark]
	fn unlock_transfer_asset_feature() {
		let acc_1 = account::<T, ()>(ACC_1);
		let target = UnlockTarget::OneselfFree;
		let feature = LockableFeature::TransferAsset;
		let season_id = <T as Config<()>>::SeasonHandler::get_current_season_id()
			.expect("Should get current season");
		unlock_season_features_for::<T, ()>(&season_id);
		let payment = T::BenchmarkHelper::create_payment_kind();

		#[extrinsic_call]
		unlock_feature(
			RawOrigin::Signed(acc_1.clone()),
			target,
			feature,
			season_id.clone(),
			Some(payment),
		);

		assert_last_event::<T, ()>(Event::FeatureUnlocked { feature, season_id, account: acc_1 });
	}

	#[benchmark]
	fn state_transition() {
		let acc_1 = account::<T, ()>(ACC_1);
		set_account_balance::<T, ()>(&acc_1, 100_u32.into());
		let season_id = <T as Config<()>>::SeasonHandler::get_current_season_id()
			.expect("Should get current season");
		let (transition_id, asset_ids) =
			T::BenchmarkHelper::create_bench_transition_for(&acc_1, &season_id, 99);
		let extra = ExtraOf::<T, ()>::default();
		let payment = T::BenchmarkHelper::create_payment_kind();

		#[extrinsic_call]
		_(RawOrigin::Signed(acc_1.clone()), transition_id.clone(), asset_ids, extra, Some(payment));

		assert_last_event::<T, ()>(Event::TransitionExecuted { account: acc_1, id: transition_id });
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
