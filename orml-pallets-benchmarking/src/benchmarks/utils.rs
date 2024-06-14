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

use frame_support::{sp_runtime::traits::StaticLookup, traits::Currency};

pub type CurrencyFor<T> = <T as orml_vesting::Config>::Currency;
pub type BalanceFor<T> =
	<CurrencyFor<T> as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type AccountIdFor<T> = <T as frame_system::Config>::AccountId;

pub fn get_account<T: frame_system::Config>() -> AccountIdFor<T> {
	account::<T>("BenchAccount")
}

fn account<T: frame_system::Config>(name: &'static str) -> T::AccountId {
	let index = 0;
	let seed = 0;
	frame_benchmarking::account(name, index, seed)
}

pub fn lookup_of_account<T: frame_system::Config>(
	who: AccountIdFor<T>,
) -> <<T as frame_system::Config>::Lookup as StaticLookup>::Source {
	<T as frame_system::Config>::Lookup::unlookup(who)
}

pub fn set_balance<T: orml_vesting::Config>(
	account: AccountIdFor<T>,
	schedule_amount: BalanceFor<T>,
) {
	CurrencyFor::<T>::make_free_balance_be(&account, schedule_amount);
}
