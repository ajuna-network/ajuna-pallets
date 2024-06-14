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

use super::utils::{
	get_account, lookup_of_account, set_balance, AccountIdFor, BalanceFor, CurrencyFor,
};
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::traits::{Currency, Get};
use frame_system::{pallet_prelude::BlockNumberFor, RawOrigin};
use sp_runtime::Saturating;
use sp_std::prelude::*;

use orml_vesting::{Call, VestingSchedule};

pub struct Pallet<T: Config>(orml_vesting::Pallet<T>);

pub trait Config: orml_vesting::Config {}

pub type Schedule<T> = VestingSchedule<BlockNumberFor<T>, BalanceFor<T>>;

const SEED: u32 = 0;

pub fn schedule<T: Config>(
	start: u32,
	period: u32,
	period_count: u32,
	per_period: BalanceFor<T>,
) -> Schedule<T> {
	Schedule::<T> { start: start.into(), period: period.into(), period_count, per_period }
}

benchmarks! {
	vested_transfer {
		let schedule = schedule::<T>(
			0, 2,3,<T>::MinVestedTransfer::get()
		);

		let from = get_account::<T>();
		let ed_times_two = CurrencyFor::<T>::minimum_balance().saturating_mul(2u32.into());
		set_balance::<T>(from.clone(), schedule.total_amount().unwrap() * ed_times_two);

		let to: AccountIdFor<T> = account("to", 0, SEED);
		let to_lookup = lookup_of_account::<T>(to.clone());
	}: _(RawOrigin::Signed(from), to_lookup, schedule.clone())
	verify {
		assert_eq!(CurrencyFor::<T>::total_balance(&to),
			schedule.total_amount().unwrap()
		);
	}

	claim {
		let i in 1 .. <T>::MaxVestingSchedules::get();

		let mut schedule = schedule::<T>(
			0, 2,3,<T>::MinVestedTransfer::get()
		);

		let from: AccountIdFor<T> = get_account::<T>();
		let ed_times_two = CurrencyFor::<T>::minimum_balance().saturating_mul(2u32.into());
		set_balance::<T>(from.clone(), schedule.total_amount().unwrap() * i.into() + ed_times_two);
		let to: AccountIdFor<T> = whitelisted_caller();
		let to_lookup = lookup_of_account::<T>(to.clone());

		for _ in 0..i {
			schedule.start = i.into();
			orml_vesting::Pallet::<T>::vested_transfer(RawOrigin::Signed(from.clone()).into(), to_lookup.clone(), schedule.clone())?;
		}
		frame_system::Pallet::<T>::set_block_number(schedule.end().unwrap() + 1u32.into());
	}: _(RawOrigin::Signed(to.clone()))
	verify {
		assert_eq!(
			CurrencyFor::<T>::free_balance(&to),
			schedule.total_amount().unwrap() * i.into(),
		);
	}

	update_vesting_schedules {
		let i in 1 .. <T>::MaxVestingSchedules::get();

		let mut schedule = schedule::<T>(
			0, 2,3,<T>::MinVestedTransfer::get()
		);

		let to: AccountIdFor<T>= account("to", 0, SEED);
		set_balance::<T>(to.clone(),schedule.total_amount().unwrap() * i.into());
		let to_lookup = lookup_of_account::<T>(to.clone());

		let mut schedules = vec![];
		for _ in 0..i {
			schedule.start = i.into();
			schedules.push(schedule.clone());
		}
	}: _(RawOrigin::Root, to_lookup, schedules)
	verify {
		assert_eq!(
			CurrencyFor::<T>::free_balance(&to),
			schedule.total_amount().unwrap() * i.into()
		);
	}

	impl_benchmark_test_suite!(
		Pallet,
		crate::mock::new_test_ext(),
		crate::mock::Runtime
	);
}
