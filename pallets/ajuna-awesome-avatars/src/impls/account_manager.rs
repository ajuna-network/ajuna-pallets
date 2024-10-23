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

use crate::*;

impl<T: Config> AccountManager for Pallet<T> {
	type AccountId = AccountIdFor<T>;

	fn is_organizer(account: &Self::AccountId) -> Result<(), DispatchError> {
		let existing_organizer = Organizer::<T>::get().ok_or(Error::<T>::OrganizerNotSet)?;
		ensure!(account == &existing_organizer, DispatchError::BadOrigin);
		Ok(())
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn set_organizer(organizer: Self::AccountId) {
		Organizer::<T>::put(organizer)
	}

	// TODO: For now we ignore the 'identifier' parameter in the AAA implementation
	fn is_whitelisted_for(_identifier: &WhitelistKey, account: &Self::AccountId) -> bool {
		WhitelistedAccounts::<T>::get().contains(account)
	}

	// TODO: For now we ignore the 'identifier' parameter in the AAA implementation
	#[cfg(feature = "runtime-benchmarks")]
	fn try_set_whitelisted_for(
		_identifier: &WhitelistKey,
		account: &Self::AccountId,
	) -> Result<(), DispatchError> {
		WhitelistedAccounts::<T>::try_mutate(|accounts| {
			accounts
				.try_push(account.clone())
				.map_err(|_| Error::<T>::WhitelistedAccountsLimitReached.into())
		})
	}
}
