use std::marker::PhantomData;

pub trait FeeHandler {
	type AccountId;
	type FeeIdentifier;
	type FeeCurrency;

	fn get_fee(account: &Self::AccountId, identifier: &Self::FeeIdentifier) -> Self::FeeCurrency;
}

impl<AccountId, Currency, T, Tid, U, Uid> FeeHandler for (T, U)
where
	T: FeeHandler<AccountId = AccountId, FeeIdentifier = Tid, FeeCurrency = Currency>,
	U: FeeHandler<AccountId = AccountId, FeeIdentifier = Uid, FeeCurrency = Currency>,
{
	type AccountId = AccountId;
	type FeeIdentifier = (Tid, Uid);
	type FeeCurrency = (Currency, Currency);

	fn get_fee(account: &Self::AccountId, identifier: &Self::FeeIdentifier) -> Self::FeeCurrency {
		(T::get_fee(account, &identifier.0), U::get_fee(account, &identifier.1))
	}
}

impl<AccountId, Currency, T, Tid, U, Uid, V, Vid> FeeHandler for (T, U, V)
where
	T: FeeHandler<AccountId = AccountId, FeeIdentifier = Tid, FeeCurrency = Currency>,
	U: FeeHandler<AccountId = AccountId, FeeIdentifier = Uid, FeeCurrency = Currency>,
	V: FeeHandler<AccountId = AccountId, FeeIdentifier = Vid, FeeCurrency = Currency>,
{
	type AccountId = AccountId;
	type FeeIdentifier = (Tid, Uid, Vid);
	type FeeCurrency = (Currency, Currency, Currency);

	fn get_fee(account: &Self::AccountId, identifier: &Self::FeeIdentifier) -> Self::FeeCurrency {
		(
			T::get_fee(account, &identifier.0),
			U::get_fee(account, &identifier.1),
			V::get_fee(account, &identifier.2),
		)
	}
}

/// Simple FeeHandler that does nothing but return the default value of 'FeeCurrency'
/// Useful if you need a partial fee setup in your runtime
pub struct DefaultFeeHandler<AccountId, FeeIdentifier, FeeCurrency: Default> {
	_phantom: PhantomData<(AccountId, FeeIdentifier, FeeCurrency)>,
}

impl<AccountId, FeeIdentifier, FeeCurrency> FeeHandler
	for DefaultFeeHandler<AccountId, FeeIdentifier, FeeCurrency>
where
	FeeCurrency: Default,
{
	type AccountId = AccountId;
	type FeeIdentifier = FeeIdentifier;
	type FeeCurrency = FeeCurrency;

	fn get_fee(_account: &Self::AccountId, _identifier: &Self::FeeIdentifier) -> Self::FeeCurrency {
		FeeCurrency::default()
	}
}
