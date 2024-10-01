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

pub mod example_transitions;
pub mod handle_fees;
pub mod primitives;
pub mod sage;
pub mod weights;

use crate::{
	example_transitions::{
		rule_asset_length_1, transition, transition_one, verify_transition_rule,
	},
	primitives::AssetT,
	sage::SageApi,
};
use core::fmt::Debug;
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use sp_core::{Decode, Encode};
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
		/// The Asset type of the game.
		///
		/// Note: This is actually tricky, we might not want to have the asset generic here, but
		/// hardcode it to a concrete type given by the game studio. In fact, I believe that this is
		/// what we need to do, otherwise the game studio can't use the concrete type in the
		/// logic they supply to the pallet here. Only, our generic helper functions and the SageApi
		/// may be generic over `AssetT`, probably. TBD.
		type Asset: AssetT + Member + Debug + Decode + Encode + TypeInfo;

		/// The fundamental Api that we can use to access all features of our
		/// game engine.
		type SageApi: SageApi<Asset = Self::Asset>;

		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The weight calculations
		type WeightInfo: WeightInfo;
	}

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

		/// A transition has been executed.
		TransitionOneExecuted {
			/// Account who initiated execution.
			account: T::AccountId,
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
		///
		/// Note:
		/// 	Is the overhead really so much bigger to add a custom call for each transition?
		#[pallet::weight(T::WeightInfo::state_transition())]
		#[pallet::call_index(0)]
		pub fn state_transition(
			origin: OriginFor<T>,
			transition_id: u32,
			assets: Vec<<T as Config>::Asset>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// Note: we can add some more elaborated error transforming here by implementing
			// From <primitives::Error> for Error<T>, this will give the possibility to return
			// granular errors to a UI.
			verify_transition_rule::<<T as Config>::SageApi>(transition_id, &assets)
				.map_err(|_e| Error::<T>::RuleNotSatisfied)?;

			transition::<<T as Config>::SageApi>(transition_id, assets)
				.map_err(|_e| Error::<T>::TransitionError)?;

			Self::deposit_event(Event::TransitionExecuted { account: sender, id: transition_id });

			Ok(())
		}

		/// Instead of matching the transition id on the inside we can expose it as a call directly.
		#[pallet::weight(T::WeightInfo::transition_one())]
		#[pallet::call_index(1)]
		pub fn transition_one(
			origin: OriginFor<T>,
			assets: Vec<<T as Config>::Asset>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// Note: we can add some more elaborated error transforming here by implementing
			// From <primitives::Error> for Error<T>, this will give the possibility to return
			// granular errors to a UI.
			rule_asset_length_1::<<T as Config>::SageApi>(&assets)
				.map_err(|_e| Error::<T>::RuleNotSatisfied)?;

			transition_one::<<T as Config>::SageApi>(assets)
				.map_err(|_e| Error::<T>::TransitionError)?;

			Self::deposit_event(Event::TransitionOneExecuted { account: sender });

			Ok(())
		}
	}
}
