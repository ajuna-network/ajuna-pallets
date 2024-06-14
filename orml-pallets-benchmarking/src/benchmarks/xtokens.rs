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
	get_vesting_account, lookup_of_account, set_balance, AccountIdFor, BalanceFor, CurrencyFor,
};
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::traits::{Currency, Get};
use frame_system::{pallet_prelude::BlockNumberFor, RawOrigin};
use sp_runtime::Saturating;
use sp_std::prelude::*;

use orml_xtokens::Call;

pub struct Pallet<T: Config>(orml_xtokens::Pallet<T>);

pub trait Config: orml_xtokens::Config {}

benchmarks! {
	transfer {
	}: _(RawOrigin::Signed(from), to_lookup, schedule.clone())
	verify {
	}

	transfer_multiasset {
	}: _(RawOrigin::Signed(from), to_lookup, schedule.clone())
	verify {
	}

	transfer_with_fee {
	}: _(RawOrigin::Signed(from), to_lookup, schedule.clone())
	verify {
	}

	transfer_multiasset_with_fee {
	}: _(RawOrigin::Signed(from), to_lookup, schedule.clone())
	verify {
	}

	transfer_multicurrencies {
	}: _(RawOrigin::Signed(from), to_lookup, schedule.clone())
	verify {
	}

	transfer_multiassets {
	}: _(RawOrigin::Signed(from), to_lookup, schedule.clone())
	verify {
	}

	impl_benchmark_test_suite!(
		Pallet,
		crate::mock::new_test_ext(),
		crate::mock::Runtime
	);
}
