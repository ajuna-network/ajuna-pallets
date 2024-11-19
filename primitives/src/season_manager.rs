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

use frame_support::{
	pallet_prelude::{Decode, DispatchError, Encode, Member, TypeInfo},
	sp_runtime::traits::AtLeast32BitUnsigned,
	Parameter,
};
use parity_scale_codec::MaxEncodedLen;
use sp_std::collections::btree_map::BTreeMap;

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq)]
pub struct SeasonFeeConfig<Balance, TransitionId> {
	/// Fee that will be deposited in the treasury when transferring an asset
	pub transfer_asset: Balance,
	/// Minimum fee that will be deposited in the treasury when buying an asset
	pub buy_asset_min: Balance,
	/// Percentage of the sell price that will be additionally deposited as fee
	/// in the treasury, if computed fee is lower than 'buy_asset' value
	/// then 'buy_asset' will be instead used as fee.
	pub buy_percent: u8,
	/// Fee that will be deposited in the treasury when upgrading an account's
	/// asset inventory
	pub upgrade_asset_inventory: Balance,
	/// Price of unlocking the `LockableFeatures::TradeAsset`
	pub unlock_trade_asset: Balance,
	/// Price of unlocking the `LockableFeatures::TransferAsset`
	pub unlock_transfer_asset: Balance,
	/// Price of executing an asset/s state transition
	pub state_transition: BTreeMap<TransitionId, Balance>,
}

impl<Balance, TransitionId> SeasonFeeConfig<Balance, TransitionId>
where
	TransitionId: PartialOrd + Ord,
	Balance: AtLeast32BitUnsigned,
{
	pub fn get_transition_fee_for(&self, transition_id: &TransitionId) -> Balance {
		self.state_transition.get(transition_id).cloned().unwrap_or(0_u32.into())
	}
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq)]
pub struct SeasonConfig<Balance, TransitionId> {
	pub fee: SeasonFeeConfig<Balance, TransitionId>,
}

pub trait SeasonManager<TransitionId> {
	type SeasonId: Member + Parameter + MaxEncodedLen;

	type AssetId: Member + Parameter + MaxEncodedLen;

	type Balance;

	fn get_season_id_for(asset_id: &Self::AssetId) -> Result<Self::SeasonId, DispatchError>;

	fn get_current_season_id() -> Self::SeasonId;

	fn is_valid_season(season_id: &Self::SeasonId) -> Result<(), DispatchError>;

	fn get_season_config_for(
		season_id: &Self::SeasonId,
	) -> Result<SeasonConfig<Self::Balance, TransitionId>, DispatchError>;
}
