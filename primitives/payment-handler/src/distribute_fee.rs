use sp_std::marker::PhantomData;

pub struct TakeNoFee<AccountId, Balance, FeeIdentifier, FeeDistribution>(
	PhantomData<(AccountId, Balance, FeeIdentifier, FeeDistribution)>,
);

impl<AccountId, Balance, FeeIdentifier, FeeDistribution> DistributeFee
	for TakeNoFee<AccountId, Balance, FeeIdentifier, FeeDistribution>
{
	type AccountId = AccountId;
	type Balance = Balance;
	type FeeIdentifier = FeeIdentifier;
	type FeeDistribution = FeeDistribution;

	fn distribute_fee(
		_: Self::Balance,
		_: &Self::AccountId,
		_: &Self::FeeIdentifier,
	) -> Option<Self::FeeDistribution> {
		None
	}
}

/// Distributes shares of a base fee to some beneficiaries.
pub trait DistributeFee {
	/// AccountId type used.
	type AccountId;

	/// Scalar balance type.
	type Balance;

	/// Fee identifier used to derive the fee distribution.
	type FeeIdentifier;

	/// Type of the fee distribution
	type FeeDistribution;

	fn distribute_fee(
		base_fee: Self::Balance,
		account: &Self::AccountId,
		identifier: &Self::FeeIdentifier,
	) -> Option<Self::FeeDistribution>;
}
