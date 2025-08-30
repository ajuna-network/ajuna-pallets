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

use crate::season_manager::SeasonConfig;
use ajuna_payment_handler::{IdentifyVoucherOrAssetId, NativeId};
use frame_support::{
	Parameter,
	pallet_prelude::{DispatchError, MaybeSerializeDeserialize, Member},
};
use parity_scale_codec::{Codec, MaxEncodedLen};

pub trait SageApi {
	type AccountId: Member + Codec;

	type AssetId: Member + Codec;

	type Asset: Member + Codec;

	type FungiblesAssetId: Clone + NativeId + IdentifyVoucherOrAssetId;

	type Balance;

	type BlockNumber;

	type SeasonId: Member + Parameter + MaxEncodedLen + MaybeSerializeDeserialize;

	type TransitionConfig;
	type HashOutput;
	fn get_transition_config() -> Self::TransitionConfig;

	fn ensure_ownership(
		owner: &Self::AccountId,
		asset_id: &Self::AssetId,
	) -> Result<Self::Asset, DispatchError>;

	fn get_asset(asset_id: &Self::AssetId) -> Result<Self::Asset, DispatchError>;

	fn create_next_asset_id() -> Option<Self::AssetId>;

	fn iter_assets_from(
		account_id: &Self::AccountId,
	) -> impl Iterator<Item = (Self::AssetId, Self::Asset)>;

	fn inspect_asset_funds(
		asset_id: &Self::AssetId,
		fungibles_asset_id: &Self::FungiblesAssetId,
	) -> Self::Balance;

	fn deposit_funds_to_asset(
		asset_id: &Self::AssetId,
		from: &Self::AccountId,
		fungibles_asset_id: Self::FungiblesAssetId,
		amount: Self::Balance,
	) -> Result<(), DispatchError>;

	fn transfer_funds_from_asset(
		asset_id: &Self::AssetId,
		to: &Self::AccountId,
		fungibles_asset_id: Self::FungiblesAssetId,
		amount: Self::Balance,
	) -> Result<(), DispatchError>;

	fn transfer_all_from_asset(
		asset_id: &Self::AssetId,
		to: &Self::AccountId,
		fungibles_asset_id: Self::FungiblesAssetId,
	) -> Result<(), DispatchError>;

	fn get_current_block_number() -> Self::BlockNumber;

	fn get_season_id_for(asset_id: &Self::AssetId) -> Result<Self::SeasonId, DispatchError>;

	fn get_current_season_id() -> Result<Self::SeasonId, DispatchError>;

	fn is_valid_season(season_id: &Self::SeasonId) -> Result<(), DispatchError>;

	fn get_season_config_for(
		season_id: &Self::SeasonId,
	) -> Result<SeasonConfig<Self::Balance>, DispatchError>;

	fn register_asset_in(
		asset_id: &Self::AssetId,
		season_id: &Self::SeasonId,
	) -> Result<(), DispatchError>;

	fn random_hash(subject: &[u8]) -> Self::HashOutput;
}
