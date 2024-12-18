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

use crate::{Pallet as Affiliates, *};
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use sp_std::prelude::*;

const ACC_1: &str = "acc_1";
const ACC_2: &str = "acc_2";
const ACC_3: &str = "acc_3";
const ACC_4: &str = "acc_4";
const ACC_5: &str = "acc_5";

fn mark_as_affiliatable<T: Config<I>, I: 'static>(account: &T::AccountId) {
	Affiliates::<T, I>::try_mark_account_as_affiliatable(account)
		.expect("Should mark as affiliatable");
}

fn affiliate_account_to<T: Config<I>, I: 'static>(
	account: &T::AccountId,
	affiliate: &T::AccountId,
) {
	<Affiliates<T, I> as AffiliateMutator<AccountIdFor<T>>>::try_add_affiliate_to(
		account, affiliate,
	)
	.expect("Should affiliate");
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
	T::AccountManager::try_add_to_whitelist(&T::WhitelistKey::get(), organizer.clone())
		.expect("Should add to whitelist");
	T::AccountManager::set_organizer(organizer);
}

#[instance_benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn enable_affiliator() {
		let acc_1 = account::<T, I>(ACC_1);
		let acc_2 = account::<T, I>(ACC_2);
		let params = T::BenchmarkHelper::create_params(1);

		#[extrinsic_call]
		_(RawOrigin::Signed(acc_1), Some(acc_2.clone()), params);

		assert_last_event::<T, I>(Event::AccountMarkedAsAffiliatable {
			account: acc_2,
			affiliate_id: 0,
		});
	}

	#[benchmark]
	fn add_affiliation() {
		let acc_1 = account::<T, I>(ACC_1);
		let acc_2 = account::<T, I>(ACC_2);
		let acc_3 = account::<T, I>(ACC_3);
		setup_organizer::<T, I>(acc_1.clone());
		mark_as_affiliatable::<T, I>(&acc_3);

		#[extrinsic_call]
		_(RawOrigin::Signed(acc_1.clone()), Some(acc_2.clone()), 0);

		assert_last_event::<T, I>(Event::AccountAffiliated { account: acc_2, to: acc_3 });
	}

	#[benchmark]
	fn remove_affiliation() {
		let acc_1 = account::<T, I>(ACC_1);
		setup_organizer::<T, I>(acc_1.clone());
		mark_as_affiliatable::<T, I>(&acc_1);
		let acc_2 = account::<T, I>(ACC_2);
		mark_as_affiliatable::<T, I>(&acc_2);
		let acc_3 = account::<T, I>(ACC_3);
		mark_as_affiliatable::<T, I>(&acc_3);
		let acc_4 = account::<T, I>(ACC_4);
		mark_as_affiliatable::<T, I>(&acc_4);
		let acc_5 = account::<T, I>(ACC_5);
		affiliate_account_to::<T, I>(&acc_1, &acc_2);
		affiliate_account_to::<T, I>(&acc_2, &acc_3);
		affiliate_account_to::<T, I>(&acc_3, &acc_4);
		affiliate_account_to::<T, I>(&acc_4, &acc_5);

		#[extrinsic_call]
		_(RawOrigin::Signed(acc_1.clone()), acc_5.clone());

		assert_last_event::<T, I>(Event::AccountUnaffiliated { account: acc_5 });
	}

	#[benchmark]
	fn set_rule_for() {
		let acc_1 = account::<T, I>(ACC_1);
		setup_organizer::<T, I>(acc_1.clone());
		let rule_id = T::BenchmarkHelper::create_rule_id(1);
		let rule = FeePropagationOf::<T, I>::try_from(vec![60, 20])
			.expect("Should create fee propagation");

		#[extrinsic_call]
		_(RawOrigin::Signed(acc_1.clone()), rule_id.clone(), rule);

		assert_last_event::<T, I>(Event::RuleAdded { rule_id });
	}

	#[benchmark]
	fn clear_rule_for() {
		let acc_1 = account::<T, I>(ACC_1);
		setup_organizer::<T, I>(acc_1.clone());
		let rule_id = T::BenchmarkHelper::create_rule_id(1);
		let rule = FeePropagationOf::<T, I>::try_from(vec![70, 15])
			.expect("Should create fee propagation");
		<Affiliates<T, I> as RuleMutator<RuleIdentifierFor<T, I>, T::AffiliateMaxLevel>>::try_add_rule_for(
			rule_id.clone(), rule
		).expect("Should be able to add rule");

		#[extrinsic_call]
		_(RawOrigin::Signed(acc_1.clone()), rule_id.clone());

		assert_last_event::<T, I>(Event::RuleCleared { rule_id });
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
