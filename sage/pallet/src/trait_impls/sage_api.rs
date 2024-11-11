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

use super::*;

/// Implement the SageApi for this instance, this is essentially where all the associated
/// types are wired together and aggregated to the API.
impl<T: Config<I>, I: 'static> SageApi for Pallet<T, I> {
	type AssetId = AssetIdOf<T, I>;
	type Asset = AssetOf<T, I>;
	type Balance = BalanceOf<T, I>;
	type AccountId = AccountIdOf<T>;

	fn ensure_ownership(
		_owner: &Self::AccountId,
		_asset: &Self::AssetId,
	) -> Result<(), SageApiError> {
		todo!()
	}

	fn try_mutate_asset<R, F: FnOnce(&mut Self::Asset) -> Result<R, SageApiError>>(
		_asset: &Self::AssetId,
		_f: F,
	) -> Result<R, SageApiError> {
		todo!()
	}

	fn transfer_ownership(_asset: Self::AssetId, _to: Self::AccountId) -> Result<(), SageApiError> {
		todo!()
	}

	fn handle_fees(_balance: Self::Balance) -> Result<(), SageApiError> {
		todo!()
	}
}
