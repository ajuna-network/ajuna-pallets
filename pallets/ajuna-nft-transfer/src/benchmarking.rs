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

use crate::{traits::IpfsUrl, *};
use ajuna_primitives::account_manager::AccountManager;
use frame_benchmarking::benchmarks;
use frame_support::{pallet_prelude::DispatchError, traits::Currency};
use frame_system::RawOrigin;
use sp_runtime::SaturatedConversion;

type CurrencyOf<T> = <T as pallet_nfts::Config>::Currency;
type CollectionIdOf<T> = <T as crate::Config>::CollectionId;
type ItemIdOf<T> = <T as crate::Config>::ItemId;

fn account<T: Config>(name: &'static str) -> T::AccountId {
	let index = 0;
	let seed = 0;
	frame_benchmarking::account(name, index, seed)
}

fn create_service_account<T: Config>() -> T::AccountId {
	let service_account = account::<T>("sa");
	ServiceAccount::<T>::put(&service_account);
	service_account
}

fn create_service_account_and_prepare_avatar<T: Config + pallet_nfts::Config>(
	player: T::AccountId,
	asset_id: ItemIdOf<T>,
) -> Result<T::AccountId, DispatchError> {
	let service_account = create_service_account::<T>();
	enable_fee_payment::<T>(&player);
	crate::Pallet::<T>::prepare_asset(RawOrigin::Signed(player).into(), asset_id)?;
	Ok(service_account)
}

fn enable_fee_payment<T: Config + pallet_nfts::Config>(player: &T::AccountId) {
	let prepare_fee = 100_000_000_000_000u128;
	CurrencyOf::<T>::make_free_balance_be(player, prepare_fee.saturated_into());
}

fn assert_last_event<T: Config>(avatars_event: Event<T>) {
	let event = <T as crate::Config>::RuntimeEvent::from(avatars_event);
	frame_system::Pallet::<T>::assert_last_event(event.into());
}

benchmarks! {
	where_clause { where T: pallet_nfts::Config }

	set_collection_id {
		let organizer = account::<T>("organizer");
		let collection_id = CollectionIdOf::<T>::from(u32::MAX);
		T::AccountManager::set_organizer(organizer.clone());
	}: _(RawOrigin::Signed(organizer), collection_id)
	verify {
		assert_last_event::<T>(Event::CollectionIdSet { collection_id })
	}

	set_service_account {
		let service_account = account::<T>("sa");
	}: _(RawOrigin::Root, service_account.clone())
	verify {
		assert_last_event::<T>(Event::<T>::ServiceAccountSet { service_account })
	}

	prepare_asset {
		let name = "player";
		let player = account::<T>(name);
		let asset_id = T::BenchmarkHelper::create_items(player.clone(), 1)[0];
		let _ = create_service_account::<T>();
		enable_fee_payment::<T>(&player);
	}: _(RawOrigin::Signed(player), asset_id)
	verify {
		assert_last_event::<T>(Event::<T>::PreparedAsset { asset_id })
	}

	unprepare_asset {
		let name = "player";
		let player = account::<T>(name);
		let asset_id = T::BenchmarkHelper::create_items(player.clone(), 1)[0];
		let _ = create_service_account_and_prepare_avatar::<T>(player.clone(), asset_id)?;
	}: _(RawOrigin::Signed(player), asset_id)
	verify {
		assert_last_event::<T>(Event::<T>::UnpreparedAsset { asset_id })
	}

	prepare_ipfs {
		let name = "player";
		let player = account::<T>(name);
		let asset_id = T::BenchmarkHelper::create_items(player.clone(), 1)[0];
		let service_account = create_service_account_and_prepare_avatar::<T>(player, asset_id)?;
		let url = IpfsUrl::try_from(b"ipfs://".to_vec()).unwrap();
	}: _(RawOrigin::Signed(service_account), asset_id, url.clone())
	verify {
		assert_last_event::<T>(Event::<T>::PreparedIpfsUrl { url })
	}

	impl_benchmark_test_suite!(
		Pallet,
		crate::mock::new_test_ext(),
		crate::mock::Test
	);
}
