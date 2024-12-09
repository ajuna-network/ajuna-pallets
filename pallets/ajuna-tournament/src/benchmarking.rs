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
		run_to_block, Balances, MinimumTournamentPhaseDuration, MockAccountManager,
		MockAssetManager, MockCategoryId, MockEntity, MockEntityId, MockRanker, RuntimeEvent,
		RuntimeOrigin, System, Test, TournamentBenchmarkHelper, TournamentPalletId1,
	},
	Pallet as Tournament, *,
};
use frame_benchmarking::benchmarks_instance_pallet;
use frame_system::RawOrigin;
use sp_runtime::BuildStorage;

impl Config for Test {
	type PalletId = TournamentPalletId1;
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type TournamentCategoryId = MockCategoryId;
	type EntityId = MockEntityId;
	type RankedEntity = MockEntity;
	type EntityRanker = MockRanker;
	type AccountManager = MockAccountManager;
	type AssetManager = MockAssetManager;
	type MinimumTournamentPhaseDuration = MinimumTournamentPhaseDuration;
	type WeightInfo = ();
	type BenchmarkHelper = TournamentBenchmarkHelper;
}

const ACC_1: &str = "acc_1";

fn create_owned_entity<T: Config<I>, I: 'static>(
	account: T::AccountId,
) -> (T::EntityId, T::RankedEntity) {
	let mut assets = T::BenchmarkHelper::create_entities(account, 1);
	assets.pop().unwrap()
}

fn account<T: Config<I>, I: 'static>(name: &'static str) -> T::AccountId {
	let index = 0;
	let seed = 0;
	frame_benchmarking::account(name, index, seed)
}

fn assert_last_event<T: Config<I>, I: 'static>(avatars_event: Event<T, I>) {
	let event = <T as Config<I>>::RuntimeEvent::from(avatars_event);
	frame_system::Pallet::<T>::assert_last_event(event.into());
}

benchmarks_instance_pallet! {
	create_tournament {
		let acc_1 = account::<T, I>(ACC_1);
		let category_id = T::BenchmarkHelper::create_category_id(1);
		let tournament_config = T::BenchmarkHelper::create_default_tournament_config();
	}: _(RawOrigin::Signed(acc_1), category_id, tournament_config)
	verify {
		assert_last_event::<T, I>(Event::TournamentCreated { category_id, tournament_id: 0 })
	}

	remove_latest_tournament {
		let acc_1 = account::<T, I>(ACC_1);
		let category_id = T::BenchmarkHelper::create_category_id(1);
		let tournament_config = T::BenchmarkHelper::create_default_tournament_config();
		Tournament::<T, I>::create_tournament(RawOrigin::Signed(acc_1.clone()).into(), category_id, tournament_config)?;
	}: _(RawOrigin::Signed(acc_1), category_id)
	verify {
		assert_last_event::<T, I>(Event::TournamentRemoved { category_id, tournament_id: 0 })
	}

	claim_tournament_reward_for {
		let acc_1 = account::<T, I>(ACC_1);
		let category_id = T::BenchmarkHelper::create_category_id(1);
		let (entity_id, entity) = create_owned_entity::<T, I>(acc_1.clone());
		let tournament_config = T::BenchmarkHelper::create_default_tournament_config();
		Tournament::<T, I>::create_tournament(RawOrigin::Signed(acc_1.clone()).into(), category_id, tournament_config)?;
		run_to_block(20);
		<Tournament<T, I> as TournamentRanker<T::TournamentCategoryId, T::RankedEntity, T::EntityId>>::try_rank_entity_in_tournament_for(
			&category_id, &entity_id, &entity
		)?;
		run_to_block(60);
	}: _(RawOrigin::Signed(acc_1.clone()), category_id, entity_id.clone())
	verify {
		assert_last_event::<T, I>(Event::RankingRewardClaimed { category_id, tournament_id: 0, entity_id ,account: acc_1 })
	}

	claim_golden_duck_for {
		let acc_1 = account::<T, I>(ACC_1);
		let category_id = T::BenchmarkHelper::create_category_id(1);
		let (entity_id, _) = create_owned_entity::<T, I>(acc_1.clone());
		let tournament_config = T::BenchmarkHelper::create_default_tournament_config();
		Tournament::<T, I>::create_tournament(RawOrigin::Signed(acc_1.clone()).into(), category_id, tournament_config)?;
		run_to_block(20);
		<Tournament<T, I> as TournamentRanker<T::TournamentCategoryId, T::RankedEntity, T::EntityId>>::try_rank_entity_for_golden_duck(
			&category_id, &entity_id
		)?;
		run_to_block(60);
	}: _(RawOrigin::Signed(acc_1.clone()), category_id, entity_id.clone())
	verify {
		assert_last_event::<T, I>(Event::GoldenDuckRewardClaimed { category_id, tournament_id: 0, entity_id ,account: acc_1 })
	}

	impl_benchmark_test_suite!(
		Tournament,
		new_test_ext(),
		Test
	);
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext.execute_with(|| {
		let acc_1 = account::<Test, ()>(ACC_1);
		Balances::force_set_balance(RuntimeOrigin::root(), acc_1.clone(), 1_000)
			.expect("Should set balance");
		MockAccountManager::set_organizer(acc_1);
	});
	ext
}
