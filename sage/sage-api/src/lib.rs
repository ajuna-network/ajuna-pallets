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
pub mod primitives;
pub mod weights;

use crate::primitives::{Asset, SageApi};
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use sp_std::prelude::*;
use weights::WeightInfo;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type SageApi: SageApi;

		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The weight calculations
		type WeightInfo: WeightInfo;
	}

	#[pallet::storage]
	pub type Creator<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A transition has been executed.
		TransitionExecuted {
			/// Account who initiated execution.
			account: T::AccountId,
			/// Transition Id that was executed.
			id: u32,
		},
	}

	/// Error for the treasury pallet.
	#[pallet::error]
	pub enum Error<T> {
		/// The rule for a given transition was not satisfied.
		RuleNotSatisfied,
		/// There was an error executing the given state transition.
		TransitionError,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Entry point for the custom state transition.
		#[pallet::weight(T::WeightInfo::state_transition())]
		#[pallet::call_index(0)]
		pub fn state_transition(
			origin: OriginFor<T>,
			transition_id: u32,
			assets: Vec<Asset>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			verify_transition_rule::<<T as Config>::SageApi>(transition_id, &assets)
				.map_err(|_e| Error::<T>::RuleNotSatisfied)?;

			transition::<<T as Config>::SageApi>(transition_id, assets)
				.map_err(|_e| Error::<T>::TransitionError)?;

			Self::deposit_event(Event::TransitionExecuted { account: sender, id: transition_id });

			Ok(())
		}
	}
}

pub fn verify_transition_rule<Sage: SageApi>(
	transition_id: u32,
	assets: &[Asset],
) -> Result<(), primitives::Error> {
	match transition_id {
		0 => rule_asset_length_1(assets),
		_ => Err(primitives::Error::InvalidTransitionId),
	}
}

pub fn transition<Sage: SageApi>(
	_transition_id: u32,
	_assets: Vec<Asset>,
) -> Result<(), primitives::Error> {
	Ok(())
}

fn rule_asset_length_1(assets: &[Asset]) -> Result<(), primitives::Error> {
	ensure!(assets.len() == 1, primitives::Error::InvalidAssetLength);
	Ok(())
}
