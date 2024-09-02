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

pub use asset::*;
pub use pallet::*;
pub use traits::*;

mod asset;
#[cfg(test)]
mod tests;
mod traits;
pub mod weights;
/*#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;*/

#[frame_support::pallet]
pub mod pallet {
	use crate::{asset::Asset, traits::*};
	use frame_support::{
		dispatch::{DispatchResultWithPostInfo, PostDispatchInfo},
		pallet_prelude::*,
		traits::{Currency, Randomness},
	};
	use frame_system::pallet_prelude::*;

	pub(crate) type AccountIdFor<T> = <T as frame_system::Config>::AccountId;
	pub(crate) type HashFor<T> = <T as frame_system::Config>::Hash;
	pub(crate) type AssetIdFor<T, I> = <<T as Config<I>>::StateMutationHandler as StateMutator<
		HashFor<T>,
		BlockNumberFor<T>,
	>>::AssetId;
	pub(crate) type AssetDataFor<T, I> = <<T as Config<I>>::StateMutationHandler as StateMutator<
		HashFor<T>,
		BlockNumberFor<T>,
	>>::AssetData;
	pub(crate) type AssetFor<T, I> = Asset<AssetDataFor<T, I>, BlockNumberFor<T>>;
	pub(crate) type MutationIdFor<T, I> = <<T as Config<I>>::StateMutationHandler as StateMutator<
		HashFor<T>,
		BlockNumberFor<T>,
	>>::MutationId;

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::config]
	pub trait Config<I: 'static = ()>: frame_system::Config {
		type RuntimeEvent: From<Event<Self, I>>
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type Currency: Currency<Self::AccountId>;

		type Randomness: Randomness<Self::Hash, BlockNumberFor<Self>>;

		/// Handler for all state change transition logic
		type StateMutationHandler: StateMutator<
			HashFor<Self>,
			BlockNumberFor<Self>,
			AccountId = AccountIdFor<Self>,
		>;

		type WeightInfo: crate::weights::WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

	#[pallet::storage]
	pub type Assets<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, AssetIdFor<T, I>, AssetFor<T, I>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		MutationExecuted { by: AccountIdFor<T>, mutation_id: MutationIdFor<T, I> },
	}

	#[pallet::error]
	pub enum Error<T, I = ()> {
		/// The request asset could not be found in storage.
		AssetNotFound,
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config<I>, I: 'static = ()> {
		_phantom: sp_std::marker::PhantomData<(T, I)>,
	}

	impl<T: Config<I>, I: 'static> Default for GenesisConfig<T, I> {
		fn default() -> Self {
			GenesisConfig { _phantom: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config<I>, I: 'static> BuildGenesisConfig for GenesisConfig<T, I> {
		fn build(&self) {}
	}

	#[pallet::hooks]
	impl<T: Config<I>, I: 'static> Hooks<BlockNumberFor<T>> for Pallet<T, I> {}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn mutate_state(
			origin: OriginFor<T>,
			mutation_id: MutationIdFor<T, I>,
			input_ids: Vec<AssetIdFor<T, I>>,
		) -> DispatchResultWithPostInfo {
			let account = ensure_signed(origin)?;

			let input_assets = input_ids
				.into_iter()
				.map(|asset_id| {
					if let Some(asset) = Assets::<T, I>::get(&asset_id) {
						Ok((asset_id.clone(), asset))
					} else {
						Err(Error::<T, I>::AssetNotFound)
					}
				})
				.collect::<Result<Vec<_>, _>>()?;

			let _result =
				T::StateMutationHandler::try_mutate_state(&account, &mutation_id, input_assets)
					.map_err(|e| e.into())?;

			Self::deposit_event(Event::<T, I>::MutationExecuted { by: account, mutation_id });

			Ok(PostDispatchInfo::from(()))
		}
	}
}
