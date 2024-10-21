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

#![cfg_attr(not(feature = "std"), no_std)]

pub mod handle_fees;
pub mod weights;

#[cfg(test)]
pub mod mock;

use frame_support::{pallet_prelude::*, traits::Currency};
use frame_system::pallet_prelude::*;
use sage_api::{AsErrorCode, SageApi, SageGameTransition};
use sp_std::prelude::*;
use weights::WeightInfo;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::pallet]
	pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

	pub type AssetOf<T, I> = <<T as Config<I>>::SageGameTransition as SageGameTransition>::Asset;
	pub type BalanceOf<T, I> = <<T as Config<I>>::Currency as Currency<AccountIdOf<T>>>::Balance;
	pub type ExtraOf<T, I> = <<T as Config<I>>::SageGameTransition as SageGameTransition>::Extra;
	pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

	#[pallet::config]
	pub trait Config<I: 'static = ()>: frame_system::Config {
		type SageGameTransition: SageGameTransition;

		type Currency: Currency<AccountIdOf<Self>>;

		/// The overarching event type.
		type RuntimeEvent: From<Event<Self, I>>
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The weight calculations
		type WeightInfo: WeightInfo;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		/// A transition has been executed.
		TransitionExecuted {
			/// Account who initiated execution.
			account: T::AccountId,
			/// Transition Id that was executed.
			id: u32,
		},

		/// A transition has been executed.
		TransitionOneExecuted {
			/// Account who initiated execution.
			account: T::AccountId,
		},
	}

	/// Error for the treasury pallet.
	#[pallet::error]
	pub enum Error<T, I = ()> {
		/// The rule for a given transition was not satisfied.
		RuleNotSatisfied,
		/// An error occurred during the state transition.
		Transition { code: u8 },
	}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		/// Entry point for the custom state transition.
		#[pallet::weight(T::WeightInfo::state_transition())]
		#[pallet::call_index(0)]
		pub fn state_transition(
			origin: OriginFor<T>,
			transition_id: u32,
			assets: Vec<AssetOf<T, I>>,
			extra: ExtraOf<T, I>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			T::SageGameTransition::verify_rule::<Self>(transition_id, &assets, &extra)
				.map_err(|_e| Error::<T, I>::RuleNotSatisfied)?;

			T::SageGameTransition::do_transition::<Self>(transition_id, assets, extra)
				.map_err(|e| Error::<T, I>::Transition { code: e.as_error_code() })?;

			Self::deposit_event(Event::TransitionExecuted { account: sender, id: transition_id });

			Ok(())
		}
	}

	/// Implement the SageApi for this instance, this is essentially where all the associated
	/// types are wired together and aggregated to the API.
	impl<T: Config<I>, I: 'static> SageApi for Pallet<T, I> {
		type Asset = AssetOf<T, I>;
		type Balance = BalanceOf<T, I>;
		type AccountId = AccountIdOf<T>;
		type Error = sage_api::Error;

		fn transfer_ownership(
			_asset: Self::Asset,
			_to: Self::AccountId,
		) -> Result<(), Self::Error> {
			todo!()
		}

		fn handle_fees(_balance: Self::Balance) -> Result<(), Self::Error> {
			todo!()
		}
	}
}
