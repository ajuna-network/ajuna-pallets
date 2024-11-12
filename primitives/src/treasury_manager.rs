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

use frame_support::{
	pallet_prelude::{DispatchError, Member},
	Parameter,
};
use parity_scale_codec::Codec;

/// The treasury manager trait that can be passed around to other pallets that need to works with
/// the treasury or its accounts
pub trait TreasuryManager {
	type AccountId: Member + Codec;

	type Currency: Member + Codec;

	type TreasuryPotKey: Parameter + Member;

	/// Is the account the assigned holder of the treasury pot for the given key
	fn is_treasurer_for(
		key: Self::TreasuryPotKey,
		account: &Self::AccountId,
	) -> Result<(), DispatchError>;

	/// Get the assigned holder of the treasury pot for the given key
	fn get_treasurer_for(key: Self::TreasuryPotKey) -> Result<Self::AccountId, DispatchError>;

	#[cfg(feature = "runtime-benchmarks")]
	fn set_treasurer_for(key: Self::TreasuryPotKey, owner: Self::AccountId);

	/// Deposit the given amount to the treasury pot for the given key
	fn deposit_into(
		depository: &Self::AccountId,
		key: &Self::TreasuryPotKey,
		fee: Self::Currency,
	) -> Result<(), DispatchError>;
}
