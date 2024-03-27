use frame_support::sp_runtime::traits::{StaticLookup};
use frame_support::traits::Currency;

pub type CurrencyFor<T> = <T as orml_vesting::Config>::Currency;
pub type BalanceFor<T> = <CurrencyFor<T> as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type AccountIdFor<T> = <T as frame_system::Config>::AccountId;

pub fn get_vesting_account<T: frame_system::Config>() -> AccountIdFor<T> {
    account::<T>("VestingAccount")
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

pub fn set_balance<T: orml_vesting::Config>(account: AccountIdFor<T>, schedule_amount: BalanceFor<T>) {
    CurrencyFor::<T>::make_free_balance_be(&account, schedule_amount);
}