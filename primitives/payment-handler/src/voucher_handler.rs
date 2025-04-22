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

use frame_support::traits::tokens::AssetId;
use sp_runtime::DispatchError;

/// Abstraction of withdrawing 'vouchers' from an account, can be used as
/// alternate payment method.
pub trait VoucherHandler {
	type AccountId;

	type Balance;

	fn consume_vouchers_from(
		account: &Self::AccountId,
		amount: Self::Balance,
	) -> Result<(), DispatchError>;
}

pub trait IdentifyVoucherOrAssetId {
	type AssetId: AssetId;

	fn is_voucher(&self) -> bool;

	fn as_asset_id(&self) -> Option<&Self::AssetId>;
}
