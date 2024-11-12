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

mod affiliates;
mod player;

use frame_support::pallet_prelude::*;

pub use affiliates::AffiliateMethods;
pub(crate) use player::*;

pub(crate) const MAX_PERCENTAGE: u8 = 100;

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, PartialEq)]
pub enum LockableFeature {
	TradeAsset,
	TransferAsset,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq)]
pub struct TransferConfig {
	pub open: bool,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq)]
pub struct TradeConfig {
	pub open: bool,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq)]
pub struct GeneralConfig<
	TransitionConfig: Member + Encode + Decode + MaxEncodedLen + TypeInfo + Default,
> {
	pub transition: TransitionConfig,
	pub transfer: TransferConfig,
	pub trade: TradeConfig,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Eq, PartialEq)]
pub enum UnlockTarget<AccountId> {
	OneselfFree,
	OneselfPaying,
	OtherPaying(AccountId),
}

pub type UnlockConfig = BoundedVec<u8, ConstU32<5>>;
