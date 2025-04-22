// Ajuna Node
// Copyright (C) 2022 BlogaTech AG
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

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
