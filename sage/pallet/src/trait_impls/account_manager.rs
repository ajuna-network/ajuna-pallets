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

use super::*;

impl<T: Config<I>, I: 'static> AccountManager for Pallet<T, I> {
	type AccountId = AccountIdOf<T>;

	fn is_organizer(account: &Self::AccountId) -> Result<(), DispatchError> {
		let existing_organizer = Organizer::<T, I>::get().ok_or(Error::<T, I>::OrganizerNotSet)?;
		ensure!(account == &existing_organizer, DispatchError::BadOrigin);
		Ok(())
	}

	fn is_whitelisted_for(_identifier: &WhitelistKey, _account: &Self::AccountId) -> bool {
		unimplemented!()
	}

	#[cfg(features = "runtime-benchmarks")]
	fn set_organizer(account: Self::AccountId) {
		Organizer::<T, I>::put(&account);
		Self::deposit_event(Event::OrganizerSet { organizer: account });
	}

	#[cfg(features = "runtime-benchmarks")]
	fn try_add_to_whitelist(
		_identifier: &WhitelistKey,
		_account: Self::AccountId,
	) -> Result<(), DispatchError> {
		unimplemented!()
	}
}
