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
		AffiliateBenchmarkHelper, AffiliateMaxLevel, AffiliateWhitelistKey, MockAccountManager,
		MockRuleId, MockRuntimeRule, RuntimeEvent, System, Test,
	},
	Pallet as Affiliates, *,
};
use ajuna_primitives::account_manager::AccountManager;
use frame_benchmarking::benchmarks_instance_pallet;
use frame_system::RawOrigin;
use sp_runtime::BuildStorage;

impl Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WhitelistKey = AffiliateWhitelistKey;
	type AccountManager = MockAccountManager;
	type RuleIdentifier = MockRuleId;
	type RuntimeRule = MockRuntimeRule;
	type AffiliateMaxLevel = AffiliateMaxLevel;
	type WeightInfo = ();
	type BenchmarkHelper = AffiliateBenchmarkHelper;
}

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

benchmarks_instance_pallet! {
	add_affiliation {
		let acc_1 = account::<T, I>(ACC_1);
		let acc_2 = account::<T, I>(ACC_2);
		let acc_3 = account::<T, I>(ACC_3);
		mark_as_affiliatable::<T, I>(&acc_3);
		let key = T::WhitelistKey::get();
		T::AccountManager::try_set_whitelisted_for(&key, &acc_1).expect("Set whitelisted");
	}: _(RawOrigin::Signed(acc_1.clone()), Some(acc_2.clone()), 0)
	verify {
		assert_last_event::<T, I>(Event::AccountAffiliated { account: acc_2, to: acc_3 })
	}

	remove_affiliation {
		let acc_1 = account::<T, I>(ACC_1);
		mark_as_affiliatable::<T, I>(&acc_1);
		let acc_2 = account::<T, I>(ACC_2);
		mark_as_affiliatable::<T, I>(&acc_2);
		let acc_3 = account::<T, I>(ACC_3);
		mark_as_affiliatable::<T, I>(&acc_3);
		let acc_4 = account::<T, I>(ACC_4);
		mark_as_affiliatable::<T, I>(&acc_4);
		let acc_5 = account::<T, I>(ACC_5);
		let key = T::WhitelistKey::get();
		affiliate_account_to::<T, I>(&acc_1, &acc_2);
		affiliate_account_to::<T, I>(&acc_2, &acc_3);
		affiliate_account_to::<T, I>(&acc_3, &acc_4);
		affiliate_account_to::<T, I>(&acc_4, &acc_5);
	}: _(RawOrigin::Signed(acc_1.clone()), acc_5.clone())
	verify {
		assert_last_event::<T, I>(Event::AccountUnaffiliated { account: acc_5 })
	}

	set_rule_for {
		let acc_1 = account::<T, I>(ACC_1);
		let rule_id = T::BenchmarkHelper::create_rule_id(1);
		let rule = T::BenchmarkHelper::create_rule(1);
	}: _(RawOrigin::Signed(acc_1.clone()), rule_id.clone(), rule)
	verify {
		assert_last_event::<T, I>(Event::RuleAdded { rule_id })
	}

	clear_rule_for {
		let acc_1 = account::<T, I>(ACC_1);
		let rule_id = T::BenchmarkHelper::create_rule_id(1);
		let rule = T::BenchmarkHelper::create_rule(1);
		<Affiliates<T, I> as RuleMutator<RuleIdentifierFor<T, I>, RuntimeRuleFor<T, I>>>::try_add_rule_for(
			rule_id.clone(), rule
		).expect("Should be able to add rule");
	}: _(RawOrigin::Signed(acc_1.clone()), rule_id.clone())
	verify {
		assert_last_event::<T, I>(Event::RuleCleared { rule_id })
	}

	impl_benchmark_test_suite!(
		Affiliates,
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
		MockAccountManager::set_organizer(acc_1);
	});
	ext
}
