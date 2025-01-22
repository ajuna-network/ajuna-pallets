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

use crate::{Pallet as Tournament, *};
use ajuna_primitives::{
	account_manager::AccountManager,
	tournament_manager::{TournamentBenchmarkHelper, TournamentRanker},
};
use frame_benchmarking::v2::*;
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use sp_arithmetic::traits::Saturating;

const ACC_1: &str = "acc_1";

fn create_owned_entity<T: Config<I>, I: 'static>(
	account: T::AccountId,
) -> (T::EntityId, T::RankedEntity) {
	let mut assets = T::BenchmarkHelper::create_entities(&account, 1);
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

fn setup_organizer<T: Config<I>, I: 'static>(organizer: T::AccountId) {
	T::AccountManager::set_organizer(organizer);
}

fn set_account_balance<T: Config<I>, I: 'static>(account: &T::AccountId, balance: BalanceOf<T, I>) {
	let _ = T::Currency::deposit_creating(account, balance);
}

fn run_to_block<T: Config<I>, I: 'static>(n: BlockNumberFor<T>) {
	while frame_system::Pallet::<T>::block_number() < n {
		let mut current_block = frame_system::Pallet::<T>::block_number();
		if current_block > 1_u32.into() {
			frame_system::Pallet::<T>::on_finalize(current_block);
			crate::Pallet::<T, I>::on_finalize(current_block);
		}
		current_block = current_block.saturating_add(1_u32.into());
		frame_system::Pallet::<T>::set_block_number(current_block);
		frame_system::Pallet::<T>::on_initialize(current_block);
		crate::Pallet::<T, I>::on_initialize(current_block);
	}
}

#[instance_benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn create_tournament() {
		let acc_1 = account::<T, I>(ACC_1);
		setup_organizer::<T, I>(acc_1.clone());
		set_account_balance::<T, I>(&acc_1, 1000_u32.into());
		let category_id = T::BenchmarkHelper::create_category_id();
		let tournament_config = T::BenchmarkHelper::create_config();

		#[extrinsic_call]
		_(RawOrigin::Signed(acc_1), category_id, tournament_config);

		assert_last_event::<T, I>(Event::TournamentCreated { category_id, tournament_id: 0 });
	}

	#[benchmark]
	fn remove_latest_tournament() {
		let acc_1 = account::<T, I>(ACC_1);
		setup_organizer::<T, I>(acc_1.clone());
		set_account_balance::<T, I>(&acc_1, 1000_u32.into());
		let category_id = T::BenchmarkHelper::create_category_id();
		let tournament_config = T::BenchmarkHelper::create_config();
		Tournament::<T, I>::create_tournament(
			RawOrigin::Signed(acc_1.clone()).into(),
			category_id,
			tournament_config,
		)
		.expect("Should create tournament");

		#[extrinsic_call]
		_(RawOrigin::Signed(acc_1), category_id);

		assert_last_event::<T, I>(Event::TournamentRemoved { category_id, tournament_id: 0 });
	}

	#[benchmark]
	fn claim_tournament_reward_for() {
		let acc_1 = account::<T, I>(ACC_1);
		setup_organizer::<T, I>(acc_1.clone());
		set_account_balance::<T, I>(&acc_1, 1000_u32.into());
		let category_id = T::BenchmarkHelper::create_category_id();
		let (entity_id, entity) = create_owned_entity::<T, I>(acc_1.clone());
		let tournament_config = T::BenchmarkHelper::create_config();
		Tournament::<T, I>::create_tournament(
			RawOrigin::Signed(acc_1.clone()).into(),
			category_id,
			tournament_config,
		)
		.expect("Should create tournament");
		run_to_block::<T, I>(20_u32.into());
		Tournament::<T, I>::try_rank_entity_in_tournament_for(&category_id, &entity_id, &entity)
			.expect("Should rank entity");
		run_to_block::<T, I>(60_u32.into());

		#[extrinsic_call]
		_(RawOrigin::Signed(acc_1.clone()), category_id, entity_id.clone());

		assert_last_event::<T, I>(Event::RankingRewardClaimed {
			category_id,
			tournament_id: 0,
			entity_id,
			account: acc_1,
		});
	}

	#[benchmark]
	fn claim_golden_duck_for() {
		let acc_1 = account::<T, I>(ACC_1);
		setup_organizer::<T, I>(acc_1.clone());
		set_account_balance::<T, I>(&acc_1, 1000_u32.into());
		let category_id = T::BenchmarkHelper::create_category_id();
		let (entity_id, _) = create_owned_entity::<T, I>(acc_1.clone());
		let tournament_config = T::BenchmarkHelper::create_config();
		Tournament::<T, I>::create_tournament(
			RawOrigin::Signed(acc_1.clone()).into(),
			category_id,
			tournament_config,
		)
		.expect("Should create tournament");
		run_to_block::<T, I>(20_u32.into());
		Tournament::<T, I>::try_rank_entity_for_golden_duck(&category_id, &entity_id)
			.expect("Should rank entity");
		run_to_block::<T, I>(60_u32.into());

		#[extrinsic_call]
		_(RawOrigin::Signed(acc_1.clone()), category_id, entity_id.clone());

		assert_last_event::<T, I>(Event::GoldenDuckRewardClaimed {
			category_id,
			tournament_id: 0,
			entity_id,
			account: acc_1,
		});
	}

	impl_benchmark_test_suite!(Tournament, crate::mock::new_test_ext(), crate::mock::Test);
}
