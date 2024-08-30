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

pub use pallet::*;
pub use traits::*;

/*#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;*/

mod asset;
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
		DefaultNoBound,
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::{CheckedAdd, One};

	pub(crate) type AccountIdFor<T> = <T as frame_system::Config>::AccountId;
	pub(crate) type AssetIdFor<T> =
		<<T as Config>::StateMutationHandler as StateMutator<BlockNumberFor<T>>>::AssetId;
	pub(crate) type AssetDataFor<T> =
		<<T as Config>::StateMutationHandler as StateMutator<BlockNumberFor<T>>>::AssetData;
	pub(crate) type AssetFor<T> = Asset<AssetDataFor<T>, BlockNumberFor<T>>;
	pub(crate) type MutationIdFor<T> =
		<<T as Config>::StateMutationHandler as StateMutator<BlockNumberFor<T>>>::MutationId;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type Currency: Currency<Self::AccountId>;

		type Randomness: Randomness<Self::Hash, BlockNumberFor<Self>>;

		/// Handler for all state change transition logic
		type StateMutationHandler: StateMutator<
			BlockNumberFor<Self>,
			AccountId = AccountIdFor<Self>,
		>;

		type WeightInfo: crate::weights::WeightInfo;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub type Something<T: Config> = StorageValue<_, u8>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		SomethingStored { block_number: BlockNumberFor<T>, who: T::AccountId },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::storage]
	pub type Assets<T: Config> = StorageMap<_, Identity, AssetIdFor<T>, AssetFor<T>>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn mutate_state(
			origin: OriginFor<T>,
			mutation_id: MutationIdFor<T>,
			input_ids: Vec<AssetIdFor<T>>,
		) -> DispatchResultWithPostInfo {
			let account = ensure_signed(origin)?;

			let input_assets = input_ids
				.into_iter()
				.map(|asset_id| {
					if let Some(asset) = Assets::<T>::get(&asset_id) {
						Ok((asset_id.clone(), asset))
					} else {
						Err(())
					}
				})
				.collect::<Result<Vec<_>, _>>()
				.expect("All assets found");

			let _result = T::StateMutationHandler::try_mutate_state(
				account,
				mutation_id,
				input_assets.as_slice(),
			).map_err(|e| e.into())?;

			Ok(PostDispatchInfo::from(()))
		}
	}
}
